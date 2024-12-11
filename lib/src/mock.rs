//! Module tasked with generating mock data to populate the application.

use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{auth, credits::Credits, user, Config, Database, ErrorKind, Result, User};

/// Generates and saves various mocking data in the database.
pub fn generate(config: &Config, db: &Database) -> Result<()> {
    user(config, db)?;

    Ok(())
}

pub fn user(config: &Config, db: &Database) -> Result<User> {
    let email = "test@mail.com".to_string();

    // does the test user already exist
    if db
        .get_collection::<User>()?
        .iter()
        .find(|u| u.email == email)
        .is_some()
        && config.dev.mock_regen != true
    {
        return Err(ErrorKind::UserWithEmailAlreadyExists(email.clone()).into());
    }

    let mut user = User::new(db)?;
    user.is_admin = true;
    user.is_disabled = false;
    user.email = email;
    user.email_confirmed = true;
    user.password_hash = Some(auth::hash_password("test")?);
    user.name = "Test User".to_string();
    user.handle = "testuser12".to_string();
    user.plan = user::Plan {
        name: "Enterprise".to_string(),
        ..Default::default()
    };
    user.credits = Credits {
        available: Decimal::ONE_HUNDRED,
        history: Default::default(),
    };
    user.notifications = user::UserNotifications {
        updates: vec![user::UpdateNotification {
            title: "Test Notification".to_string(),
            description: "Fake notification created when spawning the test user".to_string(),
            r#type: user::UpdateNotificationType::Info,
            time: Utc::now(),
            url: "/".to_string(),
            read: false,
        }],
        alerts: vec![],
    };
    user.settings = Default::default();
    user.activities = user::UserActivities {
        list: vec![user::UserActivity {
            time: Utc::now(),
            category: user::UserActivityCategory::Payment,
            message: "Paid some fake amount of money".to_string(),
        }],
    };

    db.set(&user)?;

    // // insert some orders assigned to the user
    // let order1 = Order {
    //     id: Uuid::new_v4(),
    //     user_id,
    //     time: Utc::now()
    //         .checked_sub_signed(chrono::Duration::hours(5))
    //         .unwrap(),
    //     mode: OrderMode::Manual,
    //     items: vec![OrderItem::CloudNodeHours(CloudNodeHours {
    //         count: 16,
    //         rate: dec!(0.130),
    //     })],
    //     payment: Payment {
    //         status: PaymentStatus::Pending,
    //     },
    // };
    // db.set("orders", &order1)?;
    // let order2 = Order {
    //     id: Uuid::new_v4(),
    //     user_id,
    //     time: Utc::now()
    //         .checked_sub_signed(chrono::Duration::hours(15))
    //         .unwrap(),
    //     mode: OrderMode::Manual,
    //     items: vec![OrderItem::CloudNodeHours(CloudNodeHours {
    //         count: 32,
    //         rate: dec!(0.0823),
    //     })],
    //     payment: Payment {
    //         status: PaymentStatus::Pending,
    //     },
    // };
    // db.set("orders", &order2)?;

    // TODO: add some other data owned by the user

    Ok(user)
}
