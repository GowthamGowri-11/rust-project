use crate::error::Result;
use crate::inference::InferenceEngine;
use crate::types::*;
use std::sync::Arc;
use tracing::debug;

/// Traffic classifier using ML
pub struct TrafficClassifier {
    inference_engine: Arc<dyn InferenceEngine>,
}

impl TrafficClassifier {
    pub fn new(inference_engine: Arc<dyn InferenceEngine>) -> Self {
        Self { inference_engine }
    }

    /// Classify traffic type
    pub async fn classify(&self, features: &[f32]) -> Result<TrafficClass> {
        debug!("Classifying traffic with {} features", features.len());
        
        let output = self.inference_engine.infer(features).await?;
        
        // Get class with highest probability
        let (class_idx, confidence) = output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let traffic_class = match class_idx {
            0 => TrafficClass::RealTime,
            1 => TrafficClass::Interactive,
            2 => TrafficClass::Streaming,
            3 => TrafficClass::BestEffort,
            4 => TrafficClass::Background,
            _ => TrafficClass::BestEffort,
        };

        Ok(traffic_class)
    }

    /// Classify with confidence scores
    pub async fn classify_with_confidence(&self, features: &[f32]) -> Result<ClassificationResult> {
        let output = self.inference_engine.infer(features).await?;
        
        let (class_idx, confidence) = output
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();

        let traffic_class = match class_idx {
            0 => TrafficClass::RealTime,
            1 => TrafficClass::Interactive,
            2 => TrafficClass::Streaming,
            3 => TrafficClass::BestEffort,
            4 => TrafficClass::Background,
            _ => TrafficClass::BestEffort,
        };

        Ok(ClassificationResult {
            traffic_class,
            confidence: *confidence,
            probabilities: output,
        })
    }
}

/// Congestion predictor using ML
pub struct CongestionPredictor {
    inference_engine: Arc<dyn InferenceEngine>,
    threshold: f32,
}

impl CongestionPredictor {
    pub fn new(inference_engine: Arc<dyn InferenceEngine>, threshold: f32) -> Self {
        Self {
            inference_engine,
            threshold,
        }
    }

    /// Predict congestion probability
    pub async fn predict(&self, features: &[f32]) -> Result<CongestionPrediction> {
        debug!("Predicting congestion with {} features", features.len());
        
        let output = self.inference_engine.infer(features).await?;
        
        // Binary classification: congested or not
        let congestion_prob = output.get(0).copied().unwrap_or(0.0);
        let is_congested = congestion_prob > self.threshold;

        Ok(CongestionPrediction {
            probability: congestion_prob,
            is_congested,
            confidence: congestion_prob.max(1.0 - congestion_prob),
            threshold: self.threshold,
        })
    }

    /// Predict congestion with time horizon
    pub async fn predict_with_horizon(
        &self,
        features: &[f32],
        horizon_seconds: u32,
    ) -> Result<CongestionPrediction> {
        // TODO: Use time-aware model
        // For now, use standard prediction
        let mut prediction = self.predict(features).await?;
        
        // Adjust confidence based on horizon
        let horizon_factor = 1.0 - (horizon_seconds as f32 / 300.0).min(0.5);
        prediction.confidence *= horizon_factor;
        
        Ok(prediction)
    }
}

/// Route scorer using ML
pub struct RouteScorer {
    inference_engine: Arc<dyn InferenceEngine>,
}

impl RouteScorer {
    pub fn new(inference_engine: Arc<dyn InferenceEngine>) -> Self {
        Self { inference_engine }
    }

    /// Score a route based on features
    pub async fn score_route(&self, route_features: &[f32]) -> Result<RouteScore> {
        debug!("Scoring route with {} features", route_features.len());
        
        let output = self.inference_engine.infer(route_features).await?;
        
        // Single output: route quality score (0.0 - 1.0)
        let score = output.get(0).copied().unwrap_or(0.5);
        
        Ok(RouteScore {
            score,
            quality: Self::score_to_quality(score),
            confidence: 0.8, // TODO: Calculate from model uncertainty
        })
    }

    /// Score multiple routes
    pub async fn score_routes(&self, routes: Vec<&[f32]>) -> Result<Vec<RouteScore>> {
        let mut scores = Vec::new();
        
        for route_features in routes {
            let score = self.score_route(route_features).await?;
            scores.push(score);
        }
        
        Ok(scores)
    }

    fn score_to_quality(score: f32) -> RouteQuality {
        match score {
            s if s >= 0.8 => RouteQuality::Excellent,
            s if s >= 0.6 => RouteQuality::Good,
            s if s >= 0.4 => RouteQuality::Fair,
            s if s >= 0.2 => RouteQuality::Poor,
            _ => RouteQuality::Bad,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrafficClass {
    RealTime,      // VoIP, gaming
    Interactive,   // Web browsing, SSH
    Streaming,     // Video, audio streaming
    BestEffort,    // General traffic
    Background,    // Bulk transfers, backups
}

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub traffic_class: TrafficClass,
    pub confidence: f32,
    pub probabilities: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct CongestionPrediction {
    pub probability: f32,
    pub is_congested: bool,
    pub confidence: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone)]
pub struct RouteScore {
    pub score: f32,
    pub quality: RouteQuality,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Bad,
}
