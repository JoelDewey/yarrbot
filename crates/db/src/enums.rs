use diesel_derive_enum::DbEnum;

/// The roles that a user may have with Yarrbot. These roles are only required
/// for those users actively interacting with Yarrbot; users that are only
/// listening for messages from Yarrbot do not need a role.
#[derive(DbEnum, Debug, Clone)]
#[PgType = "user_role"]
#[DieselType = "User_role"]
pub enum UserRole {
    /// Can modify other [UserRole::Administrator] users and their content. May also
    /// modify webhooks and associated rooms.
    SystemAdministrator,

    /// Users can modify webhooks and the rooms that messages from the webhooks are
    /// relayed to.
    Administrator,
}

/// The *arr that the webhook belongs to.
#[derive(DbEnum, Debug, Clone)]
#[PgType = "arr_type"]
#[DieselType = "Arr_type"]
pub enum ArrType {
    Sonarr,
    Radarr,
}
