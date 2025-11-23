use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{CreateExample, Example, ExampleResponse, UpdateExample};
use crate::errors::AppError;

pub async fn create_example(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    data: web::Json<CreateExample>,
) -> Result<HttpResponse, AppError> {
    let example = sqlx::query_as::<_, Example>(
        r#"
        INSERT INTO examples (id, title, description, user_id, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(&data.title)
    .bind(&data.description)
    .bind(*user_id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ExampleResponse::from(example)))
}

pub async fn get_examples(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let examples = sqlx::query_as::<_, Example>(
        "SELECT * FROM examples WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(*user_id)
    .fetch_all(pool.get_ref())
    .await?;

    let responses: Vec<ExampleResponse> = examples.into_iter().map(ExampleResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_example(
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (user_id, example_id) = path.into_inner();
    
    let example = sqlx::query_as::<_, Example>(
        "SELECT * FROM examples WHERE id = $1 AND user_id = $2"
    )
    .bind(example_id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Example not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ExampleResponse::from(example)))
}

pub async fn update_example(
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
    data: web::Json<UpdateExample>,
) -> Result<HttpResponse, AppError> {
    let (user_id, example_id) = path.into_inner();

    let example = sqlx::query_as::<_, Example>(
        r#"
        UPDATE examples
        SET title = COALESCE($1, title),
            description = COALESCE($2, description),
            updated_at = NOW()
        WHERE id = $3 AND user_id = $4
        RETURNING *
        "#
    )
    .bind(&data.title)
    .bind(&data.description)
    .bind(example_id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Example not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ExampleResponse::from(example)))
}

pub async fn delete_example(
    pool: web::Data<DbPool>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (user_id, example_id) = path.into_inner();

    let rows_affected = sqlx::query(
        "DELETE FROM examples WHERE id = $1 AND user_id = $2"
    )
    .bind(example_id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("Example not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}

