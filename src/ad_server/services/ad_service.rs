use crate::common::errors::AppError;
use crate::common::utils::hash_user_id;
use crate::rec_engine::engine::{RecommendationEngine, RecommendationMetrics};
use crate::schemas::internal::{AdCandidate, UserContext};
use crate::schemas::request::AdRequest;
use std::sync::Arc;
use tracing::info;

pub struct AdService {
    engine: Arc<RecommendationEngine>,
}

impl AdService {
    pub fn new(engine: Arc<RecommendationEngine>) -> Self {
        Self { engine }
    }

    pub async fn serve_ads(
        &self,
        request: &AdRequest,
        request_id: &str,
    ) -> Result<(Vec<AdCandidate>, RecommendationMetrics), AppError> {
        let user_context = self.build_user_context(request);

        let (candidates, metrics) = self
            .engine
            .recommend(&user_context, &request.slot_id, request.num_ads)
            .await?;

        info!(
            request_id = %request_id,
            retrieval_count = metrics.retrieval_count,
            final_count = metrics.final_count,
            total_ms = metrics.total_ms,
            "Ad serving completed"
        );

        Ok((candidates, metrics))
    }

    fn build_user_context(&self, request: &AdRequest) -> UserContext {
        let user_hash = request.user_id.as_deref().map(hash_user_id).unwrap_or(0);

        let mut ctx = UserContext {
            user_id: request.user_id.clone(),
            user_hash,
            os: request.device.os.clone(),
            os_version: request.device.os_version.clone().unwrap_or_default(),
            device_model: request.device.model.clone().unwrap_or_default(),
            device_brand: request.device.brand.clone().unwrap_or_default(),
            ..Default::default()
        };

        if let Some(geo) = &request.geo {
            ctx.ip = geo.ip.clone().unwrap_or_default();
            ctx.country = geo.country.clone().unwrap_or_default();
            ctx.region = geo.region.clone().unwrap_or_default();
            ctx.city = geo.city.clone().unwrap_or_default();
            ctx.latitude = geo.latitude;
            ctx.longitude = geo.longitude;
        }

        if let Some(context) = &request.context {
            ctx.app_id = context.app_id.clone().unwrap_or_default();
            ctx.app_name = context.app_name.clone().unwrap_or_default();
            ctx.network = context.network.clone().unwrap_or_default();
            ctx.carrier = context.carrier.clone().unwrap_or_default();
        }

        if let Some(uf) = &request.user_features {
            ctx.age = uf.age;
            ctx.gender = uf.gender.clone();
            ctx.interests = uf.interests.clone().unwrap_or_default();
            ctx.app_categories = uf.app_categories.clone().unwrap_or_default();
            ctx.custom_features = uf.custom.clone().unwrap_or_default();
        }

        ctx
    }

    pub async fn refresh_cache(&self) -> Result<(), AppError> {
        self.engine.refresh_cache().await
    }
}
