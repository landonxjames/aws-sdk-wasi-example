#[allow(warnings)]
mod bindings;

use std::collections::HashMap;

use aws_config::BehaviorVersion;
use aws_config::Region;
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
                .map_err(|e| data_uploader::Error::S3Error(e.to_string()))
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
            return Err(data_uploader::Error::DdbError(
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
                .map_err(|e| data_uploader::Error::DdbError(e.to_string()))
        })?;

        Ok(data_uploader::Confirmation { location: s3_uri })
    }
}

bindings::export!(Component with_types_in bindings);
