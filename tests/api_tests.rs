mod common;

use common::spawn_app;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn create_returns_201_and_the_note() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;

    let res = app
        .as_user(alice)
        .post("/notes")
        .json(&json!({ "title": "Pointing model", "body": "draft" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["title"], "Pointing model");
    assert_eq!(body["version"], 1);
    assert!(body["team_id"].is_null());
}

#[tokio::test]
async fn a_stranger_gets_404_not_403() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;
    let bob = app.seed_user("bob").await;
    let id = app.create_note(alice, "private").await;

    let res = app.as_user(bob).get(&format!("/notes/{id}")).send().await.unwrap();

    assert_eq!(res.status(), 404, "existence of a private note must not leak");
}

#[tokio::test]
async fn missing_bearer_token_is_401() {
    let app = spawn_app().await;

    let res = app
        .client
        .get(format!("{}/notes", app.address))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn unknown_user_id_is_401() {
    let app = spawn_app().await;

    let res = app
        .as_user(Uuid::new_v4())
        .get("/notes")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn empty_title_is_422() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;

    let res = app
        .as_user(alice)
        .post("/notes")
        .json(&json!({ "title": "   " }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 422);
}

#[tokio::test]
async fn patch_leaves_absent_fields_alone() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;
    let id = app.create_note(alice, "original").await;

    let res = app
        .as_user(alice)
        .patch(&format!("/notes/{id}"))
        .json(&json!({ "body": "new body" }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["title"], "original");
    assert_eq!(body["body"], "new body");
    assert_eq!(body["version"], 2);
}

#[tokio::test]
async fn deleted_notes_disappear_from_reads() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;
    let id = app.create_note(alice, "temporary").await;

    let del = app.as_user(alice).delete(&format!("/notes/{id}")).send().await.unwrap();
    assert_eq!(del.status(), 204);

    let get = app.as_user(alice).get(&format!("/notes/{id}")).send().await.unwrap();
    assert_eq!(get.status(), 404);

    let list: serde_json::Value = app
        .as_user(alice)
        .get("/notes")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn creator_becomes_an_admin() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;

    let res = app
        .as_user(alice)
        .post("/teams")
        .json(&json!({ "name": "Optics Crew" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let team: serde_json::Value = res.json().await.unwrap();
    assert_eq!(team["slug"], "optics-crew");

    let members: serde_json::Value = app
        .as_user(alice)
        .get(&format!("/teams/{}/members", team["id"].as_str().unwrap()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(members[0]["role"], "admin");
}

#[tokio::test]
async fn non_members_cannot_see_a_team() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;
    let bob = app.seed_user("bob").await;
    let team = app.seed_team_via_api(alice, "Optics Crew").await;

    let res = app
        .as_user(bob)
        .get(&format!("/teams/{team}/members"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn plain_members_cannot_add_members() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;
    let bob = app.seed_user("bob").await;
    let carol = app.seed_user("carol").await;
    let team = app.seed_team_via_api(alice, "Optics Crew").await;
    app.add_member(alice, team, bob).await;

    let res = app
        .as_user(bob)
        .post(&format!("/teams/{team}/members"))
        .json(&json!({ "user_id": carol }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403, "member knows the team exists, so 403 not 404");
}

#[tokio::test]
async fn duplicate_slug_is_409() {
    let app = spawn_app().await;
    let alice = app.seed_user("alice").await;
    app.seed_team_via_api(alice, "Optics Crew").await;

    let res = app
        .as_user(alice)
        .post("/teams")
        .json(&json!({ "name": "optics crew" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
}
