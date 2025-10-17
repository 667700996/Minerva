//! High-level orchestrator coordinating controller, vision, and engine.

use async_trait::async_trait;
use minerva_controller::{
    formation_action, formation_confirm_action, start_flow_action, DeviceController,
};
use minerva_engine::GameEngine;
use minerva_network::RealtimeServer;
use minerva_ops::{ensure_telemetry_dir, init_tracing, TelemetryStore};
use minerva_types::{
    board::BoardDiff,
    config::{MinervaConfig, OrchestratorConfig},
    events::{EngineEvent, EventKind, EventPayload, LifecycleEvent, LifecyclePhase, SystemEvent},
    game::{GameSnapshot, Move, TurnContext},
    telemetry::EngineMetrics,
    ui::{FormationPreset, StartFlowStep},
    vision::ImageFrame,
    MinervaError, Result,
};
use minerva_vision::{BoardRecognizer, RecognitionHints};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub struct Orchestrator<C, V, E, N>
where
    C: DeviceController,
    V: BoardRecognizer,
    E: GameEngine,
    N: RealtimeServer,
{
    controller: C,
    recognizer: V,
    engine: E,
    network: N,
    telemetry: TelemetryStore,
    config: OrchestratorConfig,
    last_snapshot: Option<GameSnapshot>,
}

impl<C, V, E, N> Orchestrator<C, V, E, N>
where
    C: DeviceController,
    V: BoardRecognizer,
    E: GameEngine,
    N: RealtimeServer,
{
    pub fn new(
        config: OrchestratorConfig,
        controller: C,
        recognizer: V,
        engine: E,
        network: N,
        telemetry: TelemetryStore,
    ) -> Self {
        Self {
            controller,
            recognizer,
            engine,
            network,
            telemetry,
            config,
            last_snapshot: None,
        }
    }

    pub async fn boot(&mut self, full_config: &MinervaConfig) -> Result<()> {
        init_tracing(&full_config.ops)?;
        ensure_telemetry_dir(&full_config.ops.telemetry_dir)?;

        self.controller.connect().await?;
        self.perform_start_sequence(self.config.formation).await?;
        self.engine.warm_up().await?;
        self.network.run().await?;

        let lifecycle = SystemEvent::new(
            EventKind::Lifecycle,
            EventPayload::Lifecycle(LifecycleEvent {
                phase: LifecyclePhase::Boot,
                details: Some("orchestrator boot complete".into()),
            }),
        );
        self.publish(lifecycle).await?;
        Ok(())
    }

    pub async fn play_turn(&mut self) -> Result<()> {
        let frame = self.controller.capture_frame().await?;
        let snapshot = self.recognize_board(&frame).await?;
        let diffs = self
            .last_snapshot
            .as_ref()
            .map(|prev| prev.board.differences(&snapshot.board))
            .unwrap_or_default();
        if !diffs.is_empty() {
            self.log_differences("opponent", &diffs);
        }
        self.last_snapshot = Some(snapshot.clone());
        let side = snapshot.board.side_to_move;
        let decision = self
            .engine
            .evaluate_position(&TurnContext { snapshot, side })
            .await?;

        if let Some(best_move) = decision.best_move.clone() {
            self.apply_move(best_move.clone()).await?;
        } else {
            warn!("Engine returned no move; skipping controller action");
        }

        if let Some(best_move) = decision.best_move.clone() {
            if let Some(ref mut stored) = self.last_snapshot {
                if let Err(err) = stored.apply_move(side, &best_move) {
                    warn!("내부 스냅샷 업데이트 실패: {err}");
                }
            }
        }

        let engine_event = SystemEvent::new(
            EventKind::EngineDecision,
            EventPayload::Engine(EngineEvent {
                metrics: EngineMetrics {
                    nodes: decision.searched_nodes,
                    depth: decision.depth,
                    nps: 0,
                    hashfull: 0.0,
                },
                best_line: decision.candidates.iter().map(|c| c.mv.clone()).collect(),
            }),
        );
        self.publish(engine_event).await?;
        Ok(())
    }

    async fn recognize_board(&mut self, frame: &ImageFrame) -> Result<GameSnapshot> {
        let hints = RecognitionHints {
            previous_snapshot: self.last_snapshot.clone(),
        };
        self.recognizer.recognize(frame, hints).await
    }

    async fn apply_move(&mut self, mv: Move) -> Result<()> {
        self.controller.tap_square(mv.from).await?;
        sleep(Duration::from_millis(30)).await;
        self.controller.tap_square(mv.to).await?;
        Ok(())
    }

    fn log_differences(&self, source: &str, diffs: &[BoardDiff]) {
        for diff in diffs {
            let before = diff
                .before
                .map(|p| format!("{:?}_{:?}", p.owner, p.kind))
                .unwrap_or_else(|| "None".into());
            let after = diff
                .after
                .map(|p| format!("{:?}_{:?}", p.owner, p.kind))
                .unwrap_or_else(|| "None".into());
            info!(
                "{} 변화: square ({}, {}) {} -> {}",
                source, diff.square.file, diff.square.rank, before, after
            );
        }
    }

    async fn publish(&self, event: SystemEvent) -> Result<()> {
        let cloned = event.clone();
        self.network.publish(event).await?;
        self.telemetry.record_event(cloned).await?;
        Ok(())
    }

    async fn perform_start_sequence(&mut self, formation: FormationPreset) -> Result<()> {
        self.controller
            .inject_actions(vec![
                start_flow_action(StartFlowStep::Apply),
                start_flow_action(StartFlowStep::ConfirmYes),
                start_flow_action(StartFlowStep::ConfirmOk),
            ])
            .await?;

        sleep(Duration::from_millis(150)).await;

        self.controller
            .inject_actions(vec![
                formation_action(formation),
                formation_confirm_action(),
            ])
            .await?;

        sleep(Duration::from_millis(150)).await;
        Ok(())
    }
}

#[async_trait]
pub trait MatchRunner {
    async fn run(&mut self) -> Result<()>;
}

#[async_trait]
impl<C, V, E, N> MatchRunner for Orchestrator<C, V, E, N>
where
    C: DeviceController + Send + Sync,
    V: BoardRecognizer + Send + Sync,
    E: GameEngine + Send + Sync,
    N: RealtimeServer + Send + Sync,
{
    async fn run(&mut self) -> Result<()> {
        let start_event = SystemEvent::new(
            EventKind::Lifecycle,
            EventPayload::Lifecycle(LifecycleEvent {
                phase: LifecyclePhase::MatchStart,
                details: Some("mock match started".into()),
            }),
        );
        self.publish(start_event).await?;

        for turn in 0..self.config.max_retries {
            info!("Executing turn {}", turn);
            self.play_turn().await?;
        }

        let end_event = SystemEvent::new(
            EventKind::Lifecycle,
            EventPayload::Lifecycle(LifecycleEvent {
                phase: LifecyclePhase::MatchEnd,
                details: Some("mock match completed".into()),
            }),
        );
        self.publish(end_event).await?;
        Ok(())
    }
}

pub fn orchestrator_error(message: impl Into<String>) -> MinervaError {
    MinervaError::Orchestrator(message.into())
}
