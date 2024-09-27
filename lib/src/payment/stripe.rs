use stripe::{Client, CreateCustomer, Customer};

use crate::Result;

impl super::Payment {
    pub async fn create_checkout_session(&self, client: stripe::Client) -> Result<()> {
        // create customer
        let customer = Customer::create(
            &client,
            CreateCustomer {
                name: Some("Alexander Lyon"),
                email: Some("test@async-stripe.com"),
                description: Some(
                    "A fake customer that is used to illustrate the examples in async-stripe.",
                ),
                metadata: Some(std::collections::HashMap::from([(
                    String::from("async-stripe"),
                    String::from("true"),
                )])),

                ..Default::default()
            },
        )
        .await
        .unwrap();

        Ok(())
    }
}
