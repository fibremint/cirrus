use tonic;

tonic::include_proto!("cirrus");

pub mod api {
    tonic::include_proto!("cirrus.api");
}

pub mod common {
    tonic::include_proto!("cirrus.common");
}
