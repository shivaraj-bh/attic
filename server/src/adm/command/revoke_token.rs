use anyhow::{anyhow, Result};
use clap::Parser;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use sea_orm::ActiveValue::Set;
use tracing::debug;

use crate::Opts;
use attic_server::config::Config;
use attic_server::database::entity::revoked_token;

/// Revoke all tokens for a user.
///
/// This will add the user (subject) to a revocation list, making all tokens for this user invalid for future use.
/// The user will be stored in the database along with the revocation timestamp.
///
/// Example:
/// $ atticadm revoke-token --sub "alice"
#[derive(Debug, Parser)]
pub struct RevokeToken {
    /// The user (subject) whose tokens to revoke.
    #[clap(long)]
    sub: String,
}

/// Revoke all tokens for a user by storing the subject in the revocation list
async fn revoke_user(database: &DatabaseConnection, subject: &str) -> Result<(), sea_orm::DbErr> {
    use chrono::Utc;

    let revoked_token = revoked_token::ActiveModel {
        subject: Set(subject.to_string()),
        revoked_at: Set(Utc::now()),
    };

    revoked_token.insert(database).await?;

    debug!("Revoked all tokens for user: {}", subject);
    Ok(())
}

pub async fn run(config: Config, opts: Opts) -> Result<()> {
    let revoke_cmd = opts.command.as_revoke_token().unwrap();

    // Connect to database
    use sea_orm::Database;
    let db = Database::connect(&config.database.url).await?;
 
    // Revoke user
    revoke_user(&db, &revoke_cmd.sub).await
        .map_err(|e| anyhow!("Failed to revoke user: {}", e))?;

    println!("✅ Revoked all tokens for user '{}' successfully", revoke_cmd.sub);
    println!("   The token is now invalid and cannot be used for authentication.");

    Ok(())
}
