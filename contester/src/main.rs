mod actors;
mod podman;
use actix::prelude::*;
use testing_service::testing_service_server::{TestingService, TestingServiceServer};
use testing_service::{
    GetTestStatusRequest, GetTestStatusResponse, SubmitCodeRequest, SubmitCodeResponse,
};
use tonic::{Request, Response, Status, transport::Server};
use tonic_reflection::server::Builder;

pub mod testing_service {
    tonic::include_proto!("testing_service");
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../proto_descriptor.bin");
}

#[derive(Default)]
pub struct MyTestingService {}

#[tonic::async_trait]
impl TestingService for MyTestingService {
    async fn submit_code(
        &self,
        _request: Request<SubmitCodeRequest>,
    ) -> Result<Response<SubmitCodeResponse>, Status> {
        // Заглушка: возвращаем фиктивный test_id
        let reply = SubmitCodeResponse {
            task_id: "dummy_test_id".to_string(),
        };
        Ok(Response::new(reply))
    }

    async fn get_test_status(
        &self,
        _request: Request<GetTestStatusRequest>,
    ) -> Result<Response<GetTestStatusResponse>, Status> {
        // Заглушка: возвращаем статус TEST_STATUS_UNSPECIFIED без результатов
        let reply = GetTestStatusResponse {
            status: 0, // TEST_STATUS_UNSPECIFIED
            test_results: vec![],
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //podman::podman_test().await;
    //let _addr = podman::MyActor.start();

    let addr = "[::1]:50051".parse()?;
    let testing_service = MyTestingService::default();
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(testing_service::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(TestingServiceServer::new(testing_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
