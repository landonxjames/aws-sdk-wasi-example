#[allow(warnings)]
mod bindings;

use aws_config::BehaviorVersion;
use aws_config::Region;
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
    fn new(
        config: bindings::exports::component::aws_sdk_wasi_example::data_uploader::ClientConfig,
    ) -> Self {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to generate tokio runtime");

        let aws_config = runtime.block_on(async {
            aws_config::defaults(BehaviorVersion::latest())
                .region(Region::new(config.region))
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
        input: bindings::exports::component::aws_sdk_wasi_example::data_uploader::Data,
    ) -> Result<
        bindings::exports::component::aws_sdk_wasi_example::data_uploader::Confirmation,
        String,
    > {
        todo!()
    }
}

bindings::export!(Component with_types_in bindings);
