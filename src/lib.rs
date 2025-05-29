#[allow(warnings)]
mod bindings;

use std::collections::HashMap;

use aws_config::BehaviorVersion;
use aws_config::Region;
use aws_sdk_dynamodb::error::DisplayErrorContext as DisplayErrorContextDdb;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::error::DisplayErrorContext as DisplayErrorContextS3;
use bindings::exports::component::aws_sdk_wasi_example::data_uploader;
use bindings::exports::component::aws_sdk_wasi_example::data_uploader::Guest;
use bindings::exports::component::aws_sdk_wasi_example::data_uploader::GuestDataUploaderClient;
struct Component;

impl Guest for Component {
    type DataUploaderClient = DataUploaderClient;
}

struct DataUploaderClient {
    s3_client: aws_sdk_s3::Client,
    ddb_client: aws_sdk_dynamodb::Client,
    bucket_name: String,
    table_name: String,
    runtime: tokio::runtime::Runtime,
}

impl GuestDataUploaderClient for DataUploaderClient {
    fn new(config: data_uploader::ClientConfig) -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to generate tokio runtime");

        let http_client = aws_smithy_wasm::wasi::WasiHttpClientBuilder::new().build();
        let sleep = aws_smithy_async::rt::sleep::TokioSleep::new();

        let aws_config = runtime.block_on(async {
            aws_config::defaults(BehaviorVersion::latest())
                .region(Region::new(config.region))
                .http_client(http_client)
                .sleep_impl(sleep)
                .load()
                .await
        });
        let s3_client = aws_sdk_s3::Client::new(&aws_config);
        let ddb_client = aws_sdk_dynamodb::Client::new(&aws_config);

        Self {
            s3_client,
            ddb_client,
            bucket_name: config.bucket_name,
            table_name: config.table_name,
            runtime,
        }
    }

    fn upload(
        &self,
        input: data_uploader::Data,
    ) -> Result<data_uploader::Confirmation, data_uploader::Error> {
        let _s3_res = self.runtime.block_on(async {
            self.s3_client
                .put_object()
                .bucket(&self.bucket_name)
                .key(&input.file_name)
                .body(input.data.into())
                .send()
                .await
                .map_err(|e| data_uploader::Error::S3Error(DisplayErrorContextS3(e).to_string()))
        })?;

        let s3_uri = format!("s3://{}/{}", self.bucket_name, input.file_name);

        let mut metadata: HashMap<String, aws_sdk_dynamodb::types::AttributeValue> = input
            .metadata
            .iter()
            .map(|(key, val)| {
                (
                    key.clone(),
                    aws_sdk_dynamodb::types::AttributeValue::S(val.clone()),
                )
            })
            .collect();

        let existing_s3_uri = metadata.insert(
            "s3_uri".into(),
            aws_sdk_dynamodb::types::AttributeValue::S(s3_uri.clone()),
        );

        if existing_s3_uri.is_some() {
            return Err(data_uploader::Error::InputError(
                "s3_uri is a reserved metadata key".into(),
            ));
        }

        let _ddb_res = self.runtime.block_on(async {
            self.ddb_client
                .put_item()
                .table_name(&self.table_name)
                .set_item(Some(metadata))
                .send()
                .await
                .map_err(|e| data_uploader::Error::DdbError(DisplayErrorContextDdb(e).to_string()))
        })?;

        Ok(data_uploader::Confirmation { s3_uri })
    }

    fn list(&self) -> Result<Vec<data_uploader::FileMetadata>, data_uploader::Error> {
        let filter_exp = "attribute_exists(s3_uri)".to_string();

        let ddb_res = self.runtime.block_on(async {
            self.ddb_client
                .scan()
                .table_name(&self.table_name)
                .filter_expression(filter_exp)
                .into_paginator()
                .send()
                .try_collect()
                .await
                .into_iter()
                .flatten()
                .map(|scan_out| {
                    // Combine all of the items into a single HashMap
                    let items = scan_out.items.unwrap_or_default();
                    let mut items: HashMap<&String, &AttributeValue> = items
                        .iter()
                        .flat_map(|item| item.iter().collect::<Vec<(&String, &AttributeValue)>>())
                        .collect();

                    #[allow(clippy::unnecessary_to_owned)]
                    let s3_uri = items
                        .remove(&"s3_uri".to_string())
                        .ok_or(data_uploader::Error::DdbError(
                            "s3_uri not found in DDB item".into(),
                        ))?
                        .as_s()
                        .map_err(|_| {
                            data_uploader::Error::DdbError("s3_uri is not a string".into())
                        })?
                        .clone();

                    let metadata = items
                        .into_iter()
                        .map(|(key, val)| {
                            let mapped_val = val.as_s().map_err(|_| {
                                data_uploader::Error::DdbError(format!(
                                    "metadata key {key} not a string"
                                ))
                            });

                            if let Err(err) = mapped_val {
                                Err(err)
                            } else {
                                Ok((key.clone(), mapped_val.unwrap().clone()))
                            }
                        })
                        .collect::<Result<Vec<(String, String)>, data_uploader::Error>>()?;

                    Ok(data_uploader::FileMetadata { s3_uri, metadata })
                })
                .collect::<Result<Vec<_>, _>>()
        });

        ddb_res
    }
}

bindings::export!(Component with_types_in bindings);
