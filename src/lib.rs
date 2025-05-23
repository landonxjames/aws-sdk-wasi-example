#[allow(warnings)]
mod bindings;

use crate::bindings::exports::component::aws_sdk_wasi_example::data_uploader::Guest;
use bindings::exports::component::aws_sdk_wasi_example::data_uploader::GuestDataUploaderClient;

struct Component;

impl Guest for Component {
    type DataUploaderClient = DataUploaderClient;
}

struct DataUploaderClient;

impl GuestDataUploaderClient for DataUploaderClient {
    fn new(
        config: bindings::exports::component::aws_sdk_wasi_example::data_uploader::ClientConfig,
    ) -> Self {
        todo!()
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
