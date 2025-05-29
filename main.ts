import * as component from "./js-bindings/aws_sdk_wasi_example.js";
import {
  ClientConfig,
  Data,
} from "./js-bindings/interfaces/component-aws-sdk-wasi-example-data-uploader.js";

let config: ClientConfig = {
  region: "us-west-2",
  bucketName: "lnj-aws-sdk-wasi-example",
  tableName: "lnj-aws-sdk-wasi-example",
};

let client = new component.dataUploader.DataUploaderClient(config);

let data: Data = {
  fileName: "test.txt",
  data: new TextEncoder().encode("Hello, World!"),
  metadata: [
    ["key1", "value1"],
    ["key2", "value2"],
  ],
};

let fileLocation = client.upload(data);
console.log("fileLocation:", fileLocation);

let uploadedFiles = client.list();
console.log("uploadedFiles:", JSON.stringify(uploadedFiles));
