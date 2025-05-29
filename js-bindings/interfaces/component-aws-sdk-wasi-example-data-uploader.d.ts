/** @module Interface component:aws-sdk-wasi-example/data-uploader **/
export interface ClientConfig {
  region: string,
  bucketName: string,
  tableName: string,
}
export interface Data {
  fileName: string,
  data: Uint8Array,
  metadata: Array<[string, string]>,
}
export interface Confirmation {
  s3Uri: string,
}
export interface FileMetadata {
  s3Uri: string,
  metadata: Array<[string, string]>,
}
export type Error = ErrorS3Error | ErrorDdbError | ErrorInputError;
export interface ErrorS3Error {
  tag: 's3-error',
  val: string,
}
export interface ErrorDdbError {
  tag: 'ddb-error',
  val: string,
}
export interface ErrorInputError {
  tag: 'input-error',
  val: string,
}

export class DataUploaderClient {
  constructor(config: ClientConfig)
  upload(input: Data): Confirmation;
  list(): Array<FileMetadata>;
}
