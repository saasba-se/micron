use rust_decimal_macros::dec;

use crate::{order::Order, Config, Database, Error, ErrorKind, Result, User};

impl super::Payment {
    /// Gets of creates a checkout session with stripe.
    ///
    /// Returns a checkout url that user can be redirected to.
    pub async fn checkout_session(
        &mut self,
        db: &Database,
        config: &Config,
        client: &stripe::Client,
    ) -> Result<String> {
        let order = db.get::<Order>(self.order)?;
        let mut user = db.get::<User>(order.user)?;

        if let Some(session_id) = &self.stripe_session_id {
            if let Ok(session) =
                stripe::CheckoutSession::retrieve(&client, &session_id.parse().unwrap(), &[]).await
            {
                if let Some(url) = session.url {
                    return Ok(url);
                }
            }
        }

        // Update user information as seen by stripe (`customer`)
        let customer_id = if let Some(ref customer_id) = user.stripe_customer_id {
            // TODO: if user information has changed since the last sync with
            // the stripe system, update that entry

            let mut customer =
                stripe::Customer::retrieve(&client, &customer_id.parse().unwrap(), &[]).await?;
            let mut update = stripe::UpdateCustomer::default();

            if let Some(name) = customer.name {
                if name != user.name {
                    update.name = Some(&user.name);
                }
            }
            if let Some(email) = customer.email {
                if email != user.email {
                    update.email = Some(&user.email);
                }
            }
            // TODO: sync more data points

            stripe::Customer::update(&client, &customer_id.parse().unwrap(), update).await?;
            customer_id.to_string()
        } else {
            // If we're still not tracking the user in question in the stripe
            // system, let's do that now
            let customer = stripe::Customer::create(
                &client,
                stripe::CreateCustomer {
                    name: Some(&user.name),
                    email: Some(&user.email),
                    metadata: Some(std::collections::HashMap::from([(
                        String::from("async-stripe"),
                        String::from("true"),
                    )])),

                    ..Default::default()
                },
            )
            .await?;
            user.stripe_customer_id = Some(customer.id.to_string());
            customer.id.to_string()
        };

        db.set(&user)?;

        // Get product and price information about the order that's being paid
        // for

        // We only need to provide stripe with prices, as they are linked to
        // specific product items anyway.
        let mut stripe_prices = vec![];

        // Make sure products we have in the order exist on the stripe side
        for product in order.items {
            let product_name = product.to_string();
            let product_id = product.id.to_string();

            let stripe_product = {
                let mut create_product = stripe::CreateProduct::new(&product_name);

                // stripe lets us use custom id when creating product, we'll
                // do just that
                create_product.id = Some(&product_id);

                create_product.metadata = Some(std::collections::HashMap::from([(
                    String::from("async-stripe"),
                    String::from("true"),
                )]));
                stripe::Product::create(&client, create_product)
                    .await
                    .unwrap()
            };

            // and add a price for it in USD
            let price = {
                let mut create_price = stripe::CreatePrice::new(stripe::Currency::USD);
                create_price.product = Some(stripe::IdOrCreate::Id(&product_id));
                create_price.metadata = Some(std::collections::HashMap::from([(
                    String::from("async-stripe"),
                    String::from("true"),
                )]));
                use rust_decimal::prelude::ToPrimitive;
                create_price.unit_amount =
                    Some((product.cost() * dec!(100)).to_i64().ok_or(Error::new(
                        ErrorKind::Other("failed converting price decimal".to_string()),
                    ))?);
                create_price.expand = &["product"];
                stripe::Price::create(&client, create_price).await.unwrap()
            };

            stripe_prices.push(price);
        }

        // Create a checkout session with stripe
        let checkout_session = {
            let mut params = stripe::CreateCheckoutSession::new();

            // TODO: the cancel and success urls should probably be
            // customizable somehow
            let url = format!("https://{}", config.domain);
            params.cancel_url = Some(&url);
            params.success_url = Some(&url);

            params.customer = Some(customer_id.parse().unwrap());

            // TODO: adjust based on product type
            params.mode = Some(stripe::CheckoutSessionMode::Payment);
            params.line_items = Some(
                stripe_prices
                    .iter()
                    .map(|price| stripe::CreateCheckoutSessionLineItems {
                        quantity: Some(1),
                        price: Some(price.id.to_string()),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            );
            stripe::CheckoutSession::create(&client, params).await?
        };

        self.stripe_session_id = Some(checkout_session.id.to_string());
        db.set(self)?;

        if let Some(url) = checkout_session.url {
            Ok(url)
        } else {
            Err(ErrorKind::Other(format!("failed getting stripe checkout session url")).into())
        }
    }
}
