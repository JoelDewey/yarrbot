use actix_web::dev::{ServiceResponse, ServiceRequest};
use actix_web::Error;
use tracing_actix_web::{RootSpanBuilder, DefaultRootSpanBuilder};
use tracing::Span;

pub struct YarrbotRootSpan;

impl RootSpanBuilder for YarrbotRootSpan {
    fn on_request_start(request: &ServiceRequest) -> Span {
        use tracing::field::Empty;

        tracing_actix_web::root_span!(
            request,
            webhook_arr_type = Empty,
            webhook_short_id = Empty,
            webhook_id = Empty
        )
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}