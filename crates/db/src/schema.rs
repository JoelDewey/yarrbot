table! {
    use crate::diesel_types::*;

    matrix_rooms (id) {
        id -> Uuid,
        room_id -> Text,
        webhook_id -> Uuid,
    }
}

table! {
    use crate::diesel_types::*;

    users (id) {
        id -> Uuid,
        service_username -> Text,
        user_role -> User_role,
    }
}

table! {
    use crate::diesel_types::*;

    webhooks (id) {
        id -> Uuid,
        arr_type -> Arr_type,
        username -> Text,
        password -> Bytea,
        user_id -> Uuid,
    }
}

joinable!(matrix_rooms -> webhooks (webhook_id));
joinable!(webhooks -> users (user_id));

allow_tables_to_appear_in_same_query!(matrix_rooms, users, webhooks,);
