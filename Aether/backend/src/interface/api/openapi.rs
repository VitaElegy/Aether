use utoipa::OpenApi;
use super::vocabulary;

#[derive(OpenApi)]
#[openapi(
    paths(
        vocabulary::save_vocabulary,
        vocabulary::list_vocabulary,
        vocabulary::delete_vocabulary,
        vocabulary::batch_delete_vocabulary,
        vocabulary::add_example,
        vocabulary::increment_query_count,
        vocabulary::toggle_importance,
        vocabulary::search_sentences,
    ),
    components(
        schemas(
            vocabulary::CreateVocabularyRequest,
            vocabulary::ExampleRequest,
            vocabulary::BatchDeleteRequest,
            vocabulary::ImportancePayload,
            vocabulary::SearchSentencesRequest,
        )
    ),
    tags(
        (name = "aether", description = "Aether Knowledge Management System API"),
        (name = "vocabulary", description = "Vocabulary management endpoints")
    )
)]
pub struct ApiDoc;
