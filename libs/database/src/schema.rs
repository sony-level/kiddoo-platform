// @generated automatically by Diesel CLI.

pub mod kidoo {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "kyc_status", schema = "kidoo"))]
        pub struct KycStatus;

        #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "user_role", schema = "kidoo"))]
        pub struct UserRole;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::KycStatus;

        kidoo.profiles (id) {
            id -> Uuid,
            user_id -> Uuid,
            #[max_length = 100]
            first_name -> Varchar,
            #[max_length = 100]
            last_name -> Varchar,
            #[max_length = 200]
            display_name -> Nullable<Varchar>,
            date_of_birth -> Nullable<Date>,
            #[max_length = 20]
            gender -> Nullable<Varchar>,
            profile_picture_url -> Nullable<Text>,
            #[max_length = 64]
            profile_picture_hash -> Nullable<Varchar>,
            bio -> Nullable<Text>,
            hourly_rate_cents -> Nullable<Int4>,
            kyc_status -> KycStatus,
            kyc_verified_at -> Nullable<Timestamptz>,
            kyc_verified_by -> Nullable<Uuid>,
            kyc_notes -> Nullable<Text>,
            kyc_expires_at -> Nullable<Date>,
            role_specific_data -> Jsonb,
            total_bookings -> Int4,
            completed_bookings -> Int4,
            average_rating -> Nullable<Numeric>,
            total_reviews -> Int4,
            cancellation_rate -> Nullable<Numeric>,
            is_visible -> Bool,
            is_featured -> Bool,
            search_rank -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::UserRole;

        kidoo.users (id) {
            id -> Uuid,
            keycloak_id -> Uuid,
            #[max_length = 255]
            email -> Varchar,
            email_verified -> Bool,
            #[max_length = 20]
            phone -> Nullable<Varchar>,
            phone_verified -> Bool,
            role -> UserRole,
            is_active -> Bool,
            is_blocked -> Bool,
            blocked_reason -> Nullable<Text>,
            blocked_at -> Nullable<Timestamptz>,
            blocked_by -> Nullable<Uuid>,
            #[max_length = 5]
            locale -> Varchar,
            #[max_length = 50]
            timezone -> Varchar,
            #[max_length = 20]
            theme -> Varchar,
            gdpr_consent -> Bool,
            gdpr_consent_date -> Nullable<Timestamptz>,
            #[max_length = 10]
            gdpr_consent_version -> Nullable<Varchar>,
            marketing_consent -> Bool,
            marketing_consent_date -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            last_login_at -> Nullable<Timestamptz>,
            deleted_at -> Nullable<Timestamptz>,
            deletion_reason -> Nullable<Text>,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(profiles, users);
}
