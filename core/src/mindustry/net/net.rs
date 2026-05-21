use std::collections::{HashMap, VecDeque};
use std::io;
use std::sync::Arc;
use std::time::Duration;

use super::host::Host;
use super::net_connection::NetConnection;
use super::packets::{
    AnnounceCallPacket, ClearObjectivesCallPacket, ClientBinaryPacketReliableCallPacket,
    ClientBinaryPacketUnreliableCallPacket, ClientPacketReliableCallPacket,
    ClientPacketUnreliableCallPacket, ClientPlanSnapshotCallPacket,
    ClientPlanSnapshotReceivedCallPacket, ClientSnapshotCallPacket, CompleteObjectiveCallPacket,
    Connect, ConnectCallPacket, ConnectConfirmCallPacket, ConnectPacket, ConstructFinishCallPacket,
    CopyToClipboardCallPacket, CreateBulletCallPacket, CreateMarkerCallPacket,
    CreateWeatherCallPacket, DebugStatusClientCallPacket, DebugStatusClientUnreliableCallPacket,
    DeconstructFinishCallPacket, DeletePlansCallPacket, DestroyPayloadCallPacket, Disconnect,
    DropItemCallPacket, EffectCallPacket, EffectCallPacket2, EffectReliableCallPacket,
    EntitySnapshotCallPacket, FollowUpMenuCallPacket, GameOverCallPacket, HiddenSnapshotCallPacket,
    HideFollowUpMenuCallPacket, HideHudTextCallPacket, InfoMessageCallPacket, InfoPopupCallPacket,
    InfoPopupCallPacket2, InfoPopupReliableCallPacket, InfoPopupReliableCallPacket2,
    InfoToastCallPacket, KickCallPacket, KickCallPacket2, KickReason, LabelCallPacket,
    LabelCallPacket2, LabelReliableCallPacket, LabelReliableCallPacket2,
    LandingPadLandedCallPacket, LogicExplosionCallPacket, MenuCallPacket, MenuChooseCallPacket,
    OpenUriCallPacket, PayloadDroppedCallPacket, PickedBuildPayloadCallPacket,
    PickedUnitPayloadCallPacket, PingCallPacket, PingLocationCallPacket, PingResponseCallPacket,
    PlayerDisconnectCallPacket, PlayerSpawnCallPacket, RemoveMarkerCallPacket,
    RemoveQueueBlockCallPacket, RemoveTileCallPacket, RemoveWorldLabelCallPacket,
    RequestBlockSnapshotCallPacket, RequestBuildPayloadCallPacket, RequestDebugStatusCallPacket,
    RequestDropPayloadCallPacket, RequestItemCallPacket, RequestUnitPayloadCallPacket,
    ResearchedCallPacket, RotateBlockCallPacket, SectorCaptureCallPacket,
    SendChatMessageCallPacket, SendMessageCallPacket, SendMessageCallPacket2,
    ServerBinaryPacketReliableCallPacket, ServerBinaryPacketUnreliableCallPacket,
    ServerPacketReliableCallPacket, ServerPacketUnreliableCallPacket, SetCameraPositionCallPacket,
    SetFlagCallPacket, SetFloorCallPacket, SetHudTextCallPacket, SetHudTextReliableCallPacket,
    SetItemCallPacket, SetItemsCallPacket, SetLiquidCallPacket, SetLiquidsCallPacket,
    SetMapAreaCallPacket, SetObjectivesCallPacket, SetOverlayCallPacket,
    SetPlayerTeamEditorCallPacket, SetPositionCallPacket, SetRuleCallPacket, SetRulesCallPacket,
    SetTeamCallPacket, SetTeamsCallPacket, SetTileBlocksCallPacket, SetTileCallPacket,
    SetTileFloorsCallPacket, SetTileItemsCallPacket, SetTileLiquidsCallPacket,
    SetTileOverlaysCallPacket, SetUnitCommandCallPacket, SetUnitStanceCallPacket,
    SoundAtCallPacket, SoundCallPacket, SpawnEffectCallPacket, StateSnapshotCallPacket,
    StreamBegin, StreamChunk, SyncVariableCallPacket, TakeItemsCallPacket, TextInputCallPacket,
    TextInputCallPacket2, TextInputResultCallPacket, TileConfigCallPacket, TileTapCallPacket,
    TraceInfoCallPacket, TransferInventoryCallPacket, TransferItemEffectCallPacket,
    TransferItemToCallPacket, TransferItemToUnitCallPacket, UnitBlockSpawnCallPacket,
    UnitBuildingControlSelectCallPacket, UnitCapDeathCallPacket, UnitClearCallPacket,
    UnitControlCallPacket, UnitDeathCallPacket, UnitDespawnCallPacket, UnitDestroyCallPacket,
    UnitEnteredPayloadCallPacket, UnitEnvDeathCallPacket, UnitSafeDeathCallPacket,
    UnitSpawnCallPacket, UnitTetherBlockSpawnedCallPacket, UpdateGameOverCallPacket,
    UpdateMarkerCallPacket, UpdateMarkerTextCallPacket, UpdateMarkerTextureCallPacket,
    WarningToastCallPacket, WorldDataBeginCallPacket,
};
use super::streamable::{StreamBuilder, Streamable};

#[derive(Debug, Clone, PartialEq)]
pub enum PacketKind {
    Connect(Connect),
    Disconnect(Disconnect),
    StreamBegin(StreamBegin),
    StreamChunk(StreamChunk),
    Streamable(Streamable),
    ConnectPacket(ConnectPacket),
    AnnounceCallPacket(AnnounceCallPacket),
    ClearObjectivesCallPacket(ClearObjectivesCallPacket),
    ClientBinaryPacketReliableCallPacket(ClientBinaryPacketReliableCallPacket),
    ClientBinaryPacketUnreliableCallPacket(ClientBinaryPacketUnreliableCallPacket),
    ClientPacketReliableCallPacket(ClientPacketReliableCallPacket),
    ClientPacketUnreliableCallPacket(ClientPacketUnreliableCallPacket),
    ClientPlanSnapshotCallPacket(ClientPlanSnapshotCallPacket),
    ClientPlanSnapshotReceivedCallPacket(ClientPlanSnapshotReceivedCallPacket),
    ClientSnapshotCallPacket(ClientSnapshotCallPacket),
    CompleteObjectiveCallPacket(CompleteObjectiveCallPacket),
    ConnectCallPacket(ConnectCallPacket),
    ConnectConfirmCallPacket(ConnectConfirmCallPacket),
    ConstructFinishCallPacket(ConstructFinishCallPacket),
    CopyToClipboardCallPacket(CopyToClipboardCallPacket),
    CreateBulletCallPacket(CreateBulletCallPacket),
    CreateMarkerCallPacket(CreateMarkerCallPacket),
    CreateWeatherCallPacket(CreateWeatherCallPacket),
    DebugStatusClientCallPacket(DebugStatusClientCallPacket),
    DebugStatusClientUnreliableCallPacket(DebugStatusClientUnreliableCallPacket),
    DeconstructFinishCallPacket(DeconstructFinishCallPacket),
    DeletePlansCallPacket(DeletePlansCallPacket),
    DestroyPayloadCallPacket(DestroyPayloadCallPacket),
    DropItemCallPacket(DropItemCallPacket),
    EffectCallPacket(EffectCallPacket),
    EffectCallPacket2(EffectCallPacket2),
    EffectReliableCallPacket(EffectReliableCallPacket),
    EntitySnapshotCallPacket(EntitySnapshotCallPacket),
    FollowUpMenuCallPacket(FollowUpMenuCallPacket),
    GameOverCallPacket(GameOverCallPacket),
    HiddenSnapshotCallPacket(HiddenSnapshotCallPacket),
    HideFollowUpMenuCallPacket(HideFollowUpMenuCallPacket),
    HideHudTextCallPacket(HideHudTextCallPacket),
    InfoMessageCallPacket(InfoMessageCallPacket),
    InfoPopupCallPacket(InfoPopupCallPacket),
    InfoPopupCallPacket2(InfoPopupCallPacket2),
    InfoPopupReliableCallPacket(InfoPopupReliableCallPacket),
    InfoPopupReliableCallPacket2(InfoPopupReliableCallPacket2),
    InfoToastCallPacket(InfoToastCallPacket),
    KickCallPacket(KickCallPacket),
    KickCallPacket2(KickCallPacket2),
    LabelCallPacket(LabelCallPacket),
    LabelCallPacket2(LabelCallPacket2),
    LabelReliableCallPacket(LabelReliableCallPacket),
    LabelReliableCallPacket2(LabelReliableCallPacket2),
    LandingPadLandedCallPacket(LandingPadLandedCallPacket),
    LogicExplosionCallPacket(LogicExplosionCallPacket),
    MenuCallPacket(MenuCallPacket),
    MenuChooseCallPacket(MenuChooseCallPacket),
    OpenUriCallPacket(OpenUriCallPacket),
    PayloadDroppedCallPacket(PayloadDroppedCallPacket),
    PickedBuildPayloadCallPacket(PickedBuildPayloadCallPacket),
    PickedUnitPayloadCallPacket(PickedUnitPayloadCallPacket),
    PingCallPacket(PingCallPacket),
    PingLocationCallPacket(PingLocationCallPacket),
    PingResponseCallPacket(PingResponseCallPacket),
    PlayerDisconnectCallPacket(PlayerDisconnectCallPacket),
    PlayerSpawnCallPacket(PlayerSpawnCallPacket),
    RemoveMarkerCallPacket(RemoveMarkerCallPacket),
    RemoveQueueBlockCallPacket(RemoveQueueBlockCallPacket),
    RemoveTileCallPacket(RemoveTileCallPacket),
    RemoveWorldLabelCallPacket(RemoveWorldLabelCallPacket),
    RequestBlockSnapshotCallPacket(RequestBlockSnapshotCallPacket),
    RequestBuildPayloadCallPacket(RequestBuildPayloadCallPacket),
    RequestDebugStatusCallPacket(RequestDebugStatusCallPacket),
    RequestDropPayloadCallPacket(RequestDropPayloadCallPacket),
    RequestItemCallPacket(RequestItemCallPacket),
    RequestUnitPayloadCallPacket(RequestUnitPayloadCallPacket),
    ResearchedCallPacket(ResearchedCallPacket),
    RotateBlockCallPacket(RotateBlockCallPacket),
    SectorCaptureCallPacket(SectorCaptureCallPacket),
    SendChatMessageCallPacket(SendChatMessageCallPacket),
    SendMessageCallPacket(SendMessageCallPacket),
    SendMessageCallPacket2(SendMessageCallPacket2),
    ServerBinaryPacketReliableCallPacket(ServerBinaryPacketReliableCallPacket),
    ServerBinaryPacketUnreliableCallPacket(ServerBinaryPacketUnreliableCallPacket),
    ServerPacketReliableCallPacket(ServerPacketReliableCallPacket),
    ServerPacketUnreliableCallPacket(ServerPacketUnreliableCallPacket),
    SetCameraPositionCallPacket(SetCameraPositionCallPacket),
    SetFlagCallPacket(SetFlagCallPacket),
    SetFloorCallPacket(SetFloorCallPacket),
    SetHudTextCallPacket(SetHudTextCallPacket),
    SetHudTextReliableCallPacket(SetHudTextReliableCallPacket),
    SetItemCallPacket(SetItemCallPacket),
    SetItemsCallPacket(SetItemsCallPacket),
    SetLiquidCallPacket(SetLiquidCallPacket),
    SetLiquidsCallPacket(SetLiquidsCallPacket),
    SetMapAreaCallPacket(SetMapAreaCallPacket),
    SetObjectivesCallPacket(SetObjectivesCallPacket),
    SetOverlayCallPacket(SetOverlayCallPacket),
    SetPlayerTeamEditorCallPacket(SetPlayerTeamEditorCallPacket),
    SetPositionCallPacket(SetPositionCallPacket),
    SetRuleCallPacket(SetRuleCallPacket),
    SetRulesCallPacket(SetRulesCallPacket),
    SetTeamCallPacket(SetTeamCallPacket),
    SetTeamsCallPacket(SetTeamsCallPacket),
    SetTileCallPacket(SetTileCallPacket),
    SetTileBlocksCallPacket(SetTileBlocksCallPacket),
    SetTileFloorsCallPacket(SetTileFloorsCallPacket),
    SetTileItemsCallPacket(SetTileItemsCallPacket),
    SetTileLiquidsCallPacket(SetTileLiquidsCallPacket),
    SetTileOverlaysCallPacket(SetTileOverlaysCallPacket),
    SetUnitCommandCallPacket(SetUnitCommandCallPacket),
    SetUnitStanceCallPacket(SetUnitStanceCallPacket),
    SoundCallPacket(SoundCallPacket),
    SoundAtCallPacket(SoundAtCallPacket),
    SpawnEffectCallPacket(SpawnEffectCallPacket),
    StateSnapshotCallPacket(StateSnapshotCallPacket),
    SyncVariableCallPacket(SyncVariableCallPacket),
    TakeItemsCallPacket(TakeItemsCallPacket),
    TextInputCallPacket(TextInputCallPacket),
    TextInputCallPacket2(TextInputCallPacket2),
    TextInputResultCallPacket(TextInputResultCallPacket),
    TileConfigCallPacket(TileConfigCallPacket),
    TileTapCallPacket(TileTapCallPacket),
    TraceInfoCallPacket(TraceInfoCallPacket),
    TransferInventoryCallPacket(TransferInventoryCallPacket),
    TransferItemEffectCallPacket(TransferItemEffectCallPacket),
    TransferItemToCallPacket(TransferItemToCallPacket),
    TransferItemToUnitCallPacket(TransferItemToUnitCallPacket),
    UnitBlockSpawnCallPacket(UnitBlockSpawnCallPacket),
    UnitBuildingControlSelectCallPacket(UnitBuildingControlSelectCallPacket),
    UnitCapDeathCallPacket(UnitCapDeathCallPacket),
    UnitClearCallPacket(UnitClearCallPacket),
    UnitControlCallPacket(UnitControlCallPacket),
    UnitDeathCallPacket(UnitDeathCallPacket),
    UnitDespawnCallPacket(UnitDespawnCallPacket),
    UnitDestroyCallPacket(UnitDestroyCallPacket),
    UnitEnteredPayloadCallPacket(UnitEnteredPayloadCallPacket),
    UnitEnvDeathCallPacket(UnitEnvDeathCallPacket),
    UnitSafeDeathCallPacket(UnitSafeDeathCallPacket),
    UnitSpawnCallPacket(UnitSpawnCallPacket),
    UnitTetherBlockSpawnedCallPacket(UnitTetherBlockSpawnedCallPacket),
    UpdateGameOverCallPacket(UpdateGameOverCallPacket),
    UpdateMarkerCallPacket(UpdateMarkerCallPacket),
    UpdateMarkerTextCallPacket(UpdateMarkerTextCallPacket),
    UpdateMarkerTextureCallPacket(UpdateMarkerTextureCallPacket),
    WarningToastCallPacket(WarningToastCallPacket),
    WorldDataBeginCallPacket(WorldDataBeginCallPacket),
    Other {
        id: u8,
        priority: i32,
        allow_client: bool,
        allow_server: bool,
    },
}

impl PacketKind {
    pub fn priority(&self) -> i32 {
        match self {
            PacketKind::Connect(_)
            | PacketKind::Disconnect(_)
            | PacketKind::ConnectPacket(_)
            | PacketKind::StreamBegin(_)
            | PacketKind::StreamChunk(_)
            | PacketKind::Streamable(_)
            | PacketKind::ClientSnapshotCallPacket(_)
            | PacketKind::ConnectConfirmCallPacket(_) => 2,
            PacketKind::ClientPlanSnapshotCallPacket(_)
            | PacketKind::ClientPlanSnapshotReceivedCallPacket(_)
            | PacketKind::EntitySnapshotCallPacket(_)
            | PacketKind::HiddenSnapshotCallPacket(_)
            | PacketKind::RequestBlockSnapshotCallPacket(_)
            | PacketKind::StateSnapshotCallPacket(_) => 0,
            PacketKind::UnitSpawnCallPacket(_) => 0,
            PacketKind::AnnounceCallPacket(_)
            | PacketKind::ClearObjectivesCallPacket(_)
            | PacketKind::ClientBinaryPacketReliableCallPacket(_)
            | PacketKind::ClientBinaryPacketUnreliableCallPacket(_)
            | PacketKind::ClientPacketReliableCallPacket(_)
            | PacketKind::ClientPacketUnreliableCallPacket(_)
            | PacketKind::CompleteObjectiveCallPacket(_)
            | PacketKind::ConnectCallPacket(_)
            | PacketKind::ConstructFinishCallPacket(_)
            | PacketKind::CopyToClipboardCallPacket(_)
            | PacketKind::CreateBulletCallPacket(_)
            | PacketKind::CreateMarkerCallPacket(_)
            | PacketKind::CreateWeatherCallPacket(_)
            | PacketKind::DeconstructFinishCallPacket(_)
            | PacketKind::DeletePlansCallPacket(_)
            | PacketKind::DestroyPayloadCallPacket(_)
            | PacketKind::DropItemCallPacket(_)
            | PacketKind::EffectCallPacket(_)
            | PacketKind::EffectCallPacket2(_)
            | PacketKind::EffectReliableCallPacket(_)
            | PacketKind::FollowUpMenuCallPacket(_)
            | PacketKind::GameOverCallPacket(_)
            | PacketKind::HideFollowUpMenuCallPacket(_)
            | PacketKind::HideHudTextCallPacket(_)
            | PacketKind::InfoMessageCallPacket(_)
            | PacketKind::InfoPopupCallPacket(_)
            | PacketKind::InfoPopupCallPacket2(_)
            | PacketKind::InfoPopupReliableCallPacket(_)
            | PacketKind::InfoPopupReliableCallPacket2(_)
            | PacketKind::InfoToastCallPacket(_)
            | PacketKind::LabelCallPacket(_)
            | PacketKind::LabelCallPacket2(_)
            | PacketKind::LabelReliableCallPacket(_)
            | PacketKind::LabelReliableCallPacket2(_)
            | PacketKind::LandingPadLandedCallPacket(_)
            | PacketKind::LogicExplosionCallPacket(_)
            | PacketKind::MenuCallPacket(_)
            | PacketKind::MenuChooseCallPacket(_)
            | PacketKind::OpenUriCallPacket(_)
            | PacketKind::PayloadDroppedCallPacket(_)
            | PacketKind::PickedBuildPayloadCallPacket(_)
            | PacketKind::PickedUnitPayloadCallPacket(_)
            | PacketKind::PingLocationCallPacket(_)
            | PacketKind::PingResponseCallPacket(_)
            | PacketKind::PlayerDisconnectCallPacket(_)
            | PacketKind::PlayerSpawnCallPacket(_)
            | PacketKind::RemoveMarkerCallPacket(_)
            | PacketKind::RemoveQueueBlockCallPacket(_)
            | PacketKind::RemoveTileCallPacket(_)
            | PacketKind::RemoveWorldLabelCallPacket(_)
            | PacketKind::RequestBuildPayloadCallPacket(_)
            | PacketKind::RequestDebugStatusCallPacket(_)
            | PacketKind::RequestDropPayloadCallPacket(_)
            | PacketKind::RequestItemCallPacket(_)
            | PacketKind::RequestUnitPayloadCallPacket(_)
            | PacketKind::ResearchedCallPacket(_)
            | PacketKind::RotateBlockCallPacket(_)
            | PacketKind::SendMessageCallPacket(_)
            | PacketKind::SendMessageCallPacket2(_)
            | PacketKind::SetCameraPositionCallPacket(_)
            | PacketKind::SetFlagCallPacket(_)
            | PacketKind::SetHudTextCallPacket(_)
            | PacketKind::SetHudTextReliableCallPacket(_)
            | PacketKind::SetMapAreaCallPacket(_)
            | PacketKind::SetPlayerTeamEditorCallPacket(_)
            | PacketKind::SetRuleCallPacket(_)
            | PacketKind::TextInputCallPacket(_)
            | PacketKind::TextInputCallPacket2(_)
            | PacketKind::TextInputResultCallPacket(_)
            | PacketKind::TileConfigCallPacket(_)
            | PacketKind::TileTapCallPacket(_)
            | PacketKind::TraceInfoCallPacket(_)
            | PacketKind::TransferInventoryCallPacket(_)
            | PacketKind::TransferItemEffectCallPacket(_)
            | PacketKind::TransferItemToCallPacket(_)
            | PacketKind::TransferItemToUnitCallPacket(_)
            | PacketKind::UnitBlockSpawnCallPacket(_)
            | PacketKind::UnitBuildingControlSelectCallPacket(_)
            | PacketKind::UnitCapDeathCallPacket(_)
            | PacketKind::UnitClearCallPacket(_)
            | PacketKind::UnitControlCallPacket(_)
            | PacketKind::UnitDeathCallPacket(_)
            | PacketKind::UnitDespawnCallPacket(_)
            | PacketKind::UnitDestroyCallPacket(_)
            | PacketKind::UnitEnteredPayloadCallPacket(_)
            | PacketKind::UnitEnvDeathCallPacket(_)
            | PacketKind::UnitSafeDeathCallPacket(_)
            | PacketKind::UnitTetherBlockSpawnedCallPacket(_)
            | PacketKind::UpdateGameOverCallPacket(_)
            | PacketKind::UpdateMarkerCallPacket(_)
            | PacketKind::UpdateMarkerTextCallPacket(_)
            | PacketKind::UpdateMarkerTextureCallPacket(_)
            | PacketKind::WarningToastCallPacket(_)
            | PacketKind::WorldDataBeginCallPacket(_) => 1,
            PacketKind::DebugStatusClientCallPacket(_)
            | PacketKind::DebugStatusClientUnreliableCallPacket(_)
            | PacketKind::KickCallPacket(_)
            | PacketKind::KickCallPacket2(_)
            | PacketKind::PingCallPacket(_) => 2,
            PacketKind::SectorCaptureCallPacket(_)
            | PacketKind::SetFloorCallPacket(_)
            | PacketKind::SetItemCallPacket(_)
            | PacketKind::SetItemsCallPacket(_)
            | PacketKind::SetLiquidCallPacket(_)
            | PacketKind::SetLiquidsCallPacket(_)
            | PacketKind::SetObjectivesCallPacket(_)
            | PacketKind::SetOverlayCallPacket(_)
            | PacketKind::SetPositionCallPacket(_)
            | PacketKind::SetRulesCallPacket(_)
            | PacketKind::SetTeamCallPacket(_)
            | PacketKind::SetTeamsCallPacket(_)
            | PacketKind::SetTileCallPacket(_)
            | PacketKind::SetTileBlocksCallPacket(_)
            | PacketKind::SetTileFloorsCallPacket(_)
            | PacketKind::SetTileItemsCallPacket(_)
            | PacketKind::SetTileLiquidsCallPacket(_)
            | PacketKind::SetTileOverlaysCallPacket(_)
            | PacketKind::SetUnitCommandCallPacket(_)
            | PacketKind::SetUnitStanceCallPacket(_)
            | PacketKind::SendChatMessageCallPacket(_)
            | PacketKind::ServerBinaryPacketReliableCallPacket(_)
            | PacketKind::ServerBinaryPacketUnreliableCallPacket(_)
            | PacketKind::ServerPacketReliableCallPacket(_)
            | PacketKind::ServerPacketUnreliableCallPacket(_)
            | PacketKind::SoundCallPacket(_)
            | PacketKind::SoundAtCallPacket(_)
            | PacketKind::SpawnEffectCallPacket(_)
            | PacketKind::SyncVariableCallPacket(_)
            | PacketKind::TakeItemsCallPacket(_) => 1,
            PacketKind::Other { priority, .. } => *priority,
        }
    }

    pub fn allow(&self, server: bool) -> bool {
        match self {
            PacketKind::Connect(_) | PacketKind::Disconnect(_) => true,
            PacketKind::StreamBegin(_) | PacketKind::StreamChunk(_) | PacketKind::Streamable(_) => {
                !server
            }
            PacketKind::ConnectPacket(_)
            | PacketKind::AnnounceCallPacket(_)
            | PacketKind::ClearObjectivesCallPacket(_)
            | PacketKind::CompleteObjectiveCallPacket(_)
            | PacketKind::CopyToClipboardCallPacket(_)
            | PacketKind::DebugStatusClientCallPacket(_)
            | PacketKind::DebugStatusClientUnreliableCallPacket(_)
            | PacketKind::HideFollowUpMenuCallPacket(_)
            | PacketKind::HideHudTextCallPacket(_)
            | PacketKind::InfoMessageCallPacket(_)
            | PacketKind::InfoToastCallPacket(_)
            | PacketKind::PlayerDisconnectCallPacket(_)
            | PacketKind::RemoveMarkerCallPacket(_)
            | PacketKind::DeletePlansCallPacket(_)
            | PacketKind::MenuChooseCallPacket(_)
            | PacketKind::PingLocationCallPacket(_)
            | PacketKind::RequestBuildPayloadCallPacket(_)
            | PacketKind::RequestDropPayloadCallPacket(_)
            | PacketKind::RequestItemCallPacket(_)
            | PacketKind::RequestUnitPayloadCallPacket(_)
            | PacketKind::RotateBlockCallPacket(_)
            | PacketKind::SetPlayerTeamEditorCallPacket(_)
            | PacketKind::SetUnitCommandCallPacket(_)
            | PacketKind::SetUnitStanceCallPacket(_) => true,
            PacketKind::SendChatMessageCallPacket(_)
            | PacketKind::ServerBinaryPacketReliableCallPacket(_)
            | PacketKind::ServerBinaryPacketUnreliableCallPacket(_)
            | PacketKind::ServerPacketReliableCallPacket(_)
            | PacketKind::ServerPacketUnreliableCallPacket(_) => server,
            PacketKind::ConnectCallPacket(_)
            | PacketKind::ConstructFinishCallPacket(_)
            | PacketKind::CreateBulletCallPacket(_)
            | PacketKind::CreateMarkerCallPacket(_)
            | PacketKind::CreateWeatherCallPacket(_)
            | PacketKind::DeconstructFinishCallPacket(_)
            | PacketKind::DestroyPayloadCallPacket(_)
            | PacketKind::EffectCallPacket(_)
            | PacketKind::EffectCallPacket2(_)
            | PacketKind::EffectReliableCallPacket(_)
            | PacketKind::EntitySnapshotCallPacket(_)
            | PacketKind::FollowUpMenuCallPacket(_)
            | PacketKind::GameOverCallPacket(_)
            | PacketKind::HiddenSnapshotCallPacket(_)
            | PacketKind::KickCallPacket(_)
            | PacketKind::KickCallPacket2(_)
            | PacketKind::PlayerSpawnCallPacket(_)
            | PacketKind::RemoveQueueBlockCallPacket(_)
            | PacketKind::RemoveTileCallPacket(_)
            | PacketKind::RemoveWorldLabelCallPacket(_)
            | PacketKind::InfoPopupCallPacket(_)
            | PacketKind::InfoPopupCallPacket2(_)
            | PacketKind::InfoPopupReliableCallPacket(_)
            | PacketKind::InfoPopupReliableCallPacket2(_)
            | PacketKind::LabelCallPacket(_)
            | PacketKind::LabelCallPacket2(_)
            | PacketKind::LabelReliableCallPacket(_)
            | PacketKind::LabelReliableCallPacket2(_)
            | PacketKind::LandingPadLandedCallPacket(_)
            | PacketKind::LogicExplosionCallPacket(_)
            | PacketKind::MenuCallPacket(_)
            | PacketKind::OpenUriCallPacket(_)
            | PacketKind::PayloadDroppedCallPacket(_)
            | PacketKind::PickedBuildPayloadCallPacket(_)
            | PacketKind::PickedUnitPayloadCallPacket(_)
            | PacketKind::ResearchedCallPacket(_)
            | PacketKind::SectorCaptureCallPacket(_)
            | PacketKind::SendMessageCallPacket(_)
            | PacketKind::SendMessageCallPacket2(_)
            | PacketKind::SetCameraPositionCallPacket(_)
            | PacketKind::SetFlagCallPacket(_)
            | PacketKind::SetFloorCallPacket(_)
            | PacketKind::SetHudTextCallPacket(_)
            | PacketKind::SetHudTextReliableCallPacket(_)
            | PacketKind::SetItemCallPacket(_)
            | PacketKind::SetItemsCallPacket(_)
            | PacketKind::SetLiquidCallPacket(_)
            | PacketKind::SetLiquidsCallPacket(_)
            | PacketKind::SetMapAreaCallPacket(_)
            | PacketKind::SetObjectivesCallPacket(_)
            | PacketKind::SetOverlayCallPacket(_)
            | PacketKind::SetPositionCallPacket(_)
            | PacketKind::SetRuleCallPacket(_)
            | PacketKind::SetRulesCallPacket(_)
            | PacketKind::SetTeamCallPacket(_)
            | PacketKind::SetTeamsCallPacket(_)
            | PacketKind::SetTileCallPacket(_)
            | PacketKind::SetTileBlocksCallPacket(_)
            | PacketKind::SetTileFloorsCallPacket(_)
            | PacketKind::SetTileItemsCallPacket(_)
            | PacketKind::SetTileLiquidsCallPacket(_)
            | PacketKind::SetTileOverlaysCallPacket(_)
            | PacketKind::SoundCallPacket(_)
            | PacketKind::SoundAtCallPacket(_)
            | PacketKind::SpawnEffectCallPacket(_)
            | PacketKind::StateSnapshotCallPacket(_)
            | PacketKind::SyncVariableCallPacket(_)
            | PacketKind::TakeItemsCallPacket(_)
            | PacketKind::TextInputResultCallPacket(_)
            | PacketKind::TileConfigCallPacket(_)
            | PacketKind::TileTapCallPacket(_)
            | PacketKind::TransferInventoryCallPacket(_)
            | PacketKind::UnitClearCallPacket(_)
            | PacketKind::UnitControlCallPacket(_) => true,
            PacketKind::TraceInfoCallPacket(_)
            | PacketKind::TransferItemEffectCallPacket(_)
            | PacketKind::TransferItemToCallPacket(_)
            | PacketKind::TransferItemToUnitCallPacket(_)
            | PacketKind::UnitBlockSpawnCallPacket(_)
            | PacketKind::UnitBuildingControlSelectCallPacket(_)
            | PacketKind::UnitCapDeathCallPacket(_)
            | PacketKind::UnitDeathCallPacket(_)
            | PacketKind::UnitDespawnCallPacket(_)
            | PacketKind::UnitDestroyCallPacket(_)
            | PacketKind::UnitEnteredPayloadCallPacket(_)
            | PacketKind::UnitEnvDeathCallPacket(_)
            | PacketKind::UnitSafeDeathCallPacket(_)
            | PacketKind::UnitSpawnCallPacket(_)
            | PacketKind::UnitTetherBlockSpawnedCallPacket(_)
            | PacketKind::UpdateGameOverCallPacket(_)
            | PacketKind::UpdateMarkerCallPacket(_)
            | PacketKind::UpdateMarkerTextCallPacket(_)
            | PacketKind::UpdateMarkerTextureCallPacket(_)
            | PacketKind::WarningToastCallPacket(_)
            | PacketKind::WorldDataBeginCallPacket(_)
            | PacketKind::TextInputCallPacket(_)
            | PacketKind::TextInputCallPacket2(_) => !server,
            PacketKind::ClientBinaryPacketReliableCallPacket(_)
            | PacketKind::ClientBinaryPacketUnreliableCallPacket(_)
            | PacketKind::ClientPacketReliableCallPacket(_)
            | PacketKind::ClientPacketUnreliableCallPacket(_)
            | PacketKind::ClientPlanSnapshotCallPacket(_)
            | PacketKind::ClientSnapshotCallPacket(_)
            | PacketKind::ConnectConfirmCallPacket(_)
            | PacketKind::DropItemCallPacket(_)
            | PacketKind::PingCallPacket(_)
            | PacketKind::RequestBlockSnapshotCallPacket(_)
            | PacketKind::RequestDebugStatusCallPacket(_) => server,
            PacketKind::PingResponseCallPacket(_) => !server,
            PacketKind::ClientPlanSnapshotReceivedCallPacket(_) => !server,
            PacketKind::Other {
                allow_client,
                allow_server,
                ..
            } => {
                if server {
                    *allow_server
                } else {
                    *allow_client
                }
            }
        }
    }
}

pub type ConnectFilter = Arc<dyn Fn(&str) -> bool + Send + Sync + 'static>;
pub type HostCallback = Box<dyn Fn(Host) + Send + 'static>;
pub type DoneCallback = Box<dyn Fn() + Send + 'static>;
pub type ClientListener = Box<dyn FnMut(&PacketKind) + Send + 'static>;
pub type ServerListener = Box<dyn FnMut(Option<i32>, &PacketKind) + Send + 'static>;
pub type ClientTypedListener<T> = Box<dyn FnMut(&T) + Send + 'static>;
pub type ServerTypedListener<T> = Box<dyn FnMut(Option<i32>, &T) + Send + 'static>;

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderEvent {
    ClientConnected {
        address_tcp: String,
    },
    ClientDisconnected {
        reason: String,
    },
    ClientPacket(PacketKind),
    ServerConnected {
        connection_id: i32,
        address: String,
    },
    ServerDisconnected {
        connection_id: i32,
        reason: String,
    },
    ServerPacket {
        connection_id: i32,
        packet: PacketKind,
    },
}

pub trait NetProvider: Send {
    fn connect_client(
        &mut self,
        ip: &str,
        port: u16,
        success: Box<dyn Fn() + Send + 'static>,
    ) -> io::Result<()>;

    fn send_client(&mut self, object: &PacketKind, reliable: bool) -> io::Result<()>;

    fn disconnect_client(&mut self);

    fn discover_servers(&self, callback: HostCallback, done: DoneCallback);

    fn ping_host(&self, address: &str, port: u16, timeout: Duration) -> io::Result<Host>;

    fn host_server(&mut self, port: u16) -> io::Result<()>;

    fn get_connections(&self) -> Vec<NetConnection>;

    fn close_server(&mut self);

    fn send_server(&mut self, _object: &PacketKind, _reliable: bool) -> io::Result<()> {
        Ok(())
    }

    fn send_server_to(
        &mut self,
        _connection_id: i32,
        object: &PacketKind,
        reliable: bool,
    ) -> io::Result<()> {
        self.send_server(object, reliable)
    }

    fn send_server_except(
        &mut self,
        _except_connection_id: i32,
        object: &PacketKind,
        reliable: bool,
    ) -> io::Result<()> {
        self.send_server(object, reliable)
    }

    fn drain_events(&mut self) -> Vec<ProviderEvent> {
        Vec::new()
    }

    fn dispose(&mut self) {
        self.disconnect_client();
        self.close_server();
    }

    fn set_connect_filter(&mut self, _connect_filter: Option<ConnectFilter>) {}

    fn get_connect_filter(&self) -> Option<ConnectFilter> {
        None
    }
}

#[derive(Default)]
pub struct NoopNetProvider {
    connect_filter: Option<ConnectFilter>,
}

impl NetProvider for NoopNetProvider {
    fn connect_client(
        &mut self,
        _ip: &str,
        _port: u16,
        _success: Box<dyn Fn() + Send + 'static>,
    ) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no NetProvider is installed",
        ))
    }

    fn send_client(&mut self, _object: &PacketKind, _reliable: bool) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "no NetProvider is installed",
        ))
    }

    fn disconnect_client(&mut self) {}

    fn discover_servers(&self, _callback: HostCallback, done: DoneCallback) {
        done();
    }

    fn ping_host(&self, _address: &str, _port: u16, _timeout: Duration) -> io::Result<Host> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no NetProvider is installed",
        ))
    }

    fn host_server(&mut self, _port: u16) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no NetProvider is installed",
        ))
    }

    fn get_connections(&self) -> Vec<NetConnection> {
        Vec::new()
    }

    fn close_server(&mut self) {}

    fn set_connect_filter(&mut self, connect_filter: Option<ConnectFilter>) {
        self.connect_filter = connect_filter;
    }

    fn get_connect_filter(&self) -> Option<ConnectFilter> {
        self.connect_filter.clone()
    }
}

pub struct Net {
    provider: Box<dyn NetProvider>,
    server: bool,
    active: bool,
    client_loaded: bool,
    current_stream: Option<StreamBuilder>,
    packet_queue: VecDeque<PacketKind>,
    streams: HashMap<i32, StreamBuilder>,
    server_connections: HashMap<i32, NetConnection>,
    client_listeners: Vec<ClientListener>,
    server_listeners: Vec<ServerListener>,
    client_connect_listeners: Vec<ClientTypedListener<Connect>>,
    client_disconnect_listeners: Vec<ClientTypedListener<Disconnect>>,
    client_world_stream_listeners: Vec<ClientTypedListener<Streamable>>,
    server_connect_listeners: Vec<ServerTypedListener<Connect>>,
    server_disconnect_listeners: Vec<ServerTypedListener<Disconnect>>,
    server_connect_packet_listeners: Vec<ServerTypedListener<ConnectPacket>>,
    server_connect_confirm_listeners: Vec<ServerTypedListener<ConnectConfirmCallPacket>>,
    handled_client_packets: Vec<PacketKind>,
    handled_server_packets: Vec<PacketKind>,
}

impl std::fmt::Debug for Net {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Net")
            .field("server", &self.server)
            .field("active", &self.active)
            .field("client_loaded", &self.client_loaded)
            .field("current_stream", &self.current_stream)
            .field("packet_queue", &self.packet_queue)
            .field("streams", &self.streams)
            .field("server_connections", &self.server_connections)
            .field("client_listeners", &self.client_listeners.len())
            .field("server_listeners", &self.server_listeners.len())
            .field(
                "client_connect_listeners",
                &self.client_connect_listeners.len(),
            )
            .field(
                "client_disconnect_listeners",
                &self.client_disconnect_listeners.len(),
            )
            .field(
                "client_world_stream_listeners",
                &self.client_world_stream_listeners.len(),
            )
            .field(
                "server_connect_listeners",
                &self.server_connect_listeners.len(),
            )
            .field(
                "server_disconnect_listeners",
                &self.server_disconnect_listeners.len(),
            )
            .field(
                "server_connect_packet_listeners",
                &self.server_connect_packet_listeners.len(),
            )
            .field(
                "server_connect_confirm_listeners",
                &self.server_connect_confirm_listeners.len(),
            )
            .field("handled_client_packets", &self.handled_client_packets)
            .field("handled_server_packets", &self.handled_server_packets)
            .finish()
    }
}

impl Default for Net {
    fn default() -> Self {
        Self::new(Box::<NoopNetProvider>::default())
    }
}

impl Net {
    pub fn new(provider: Box<dyn NetProvider>) -> Self {
        Self {
            provider,
            server: false,
            active: false,
            client_loaded: false,
            current_stream: None,
            packet_queue: VecDeque::new(),
            streams: HashMap::new(),
            server_connections: HashMap::new(),
            client_listeners: Vec::new(),
            server_listeners: Vec::new(),
            client_connect_listeners: Vec::new(),
            client_disconnect_listeners: Vec::new(),
            client_world_stream_listeners: Vec::new(),
            server_connect_listeners: Vec::new(),
            server_disconnect_listeners: Vec::new(),
            server_connect_packet_listeners: Vec::new(),
            server_connect_confirm_listeners: Vec::new(),
            handled_client_packets: Vec::new(),
            handled_server_packets: Vec::new(),
        }
    }

    pub fn set_provider(&mut self, provider: Box<dyn NetProvider>) {
        self.provider.dispose();
        self.provider = provider;
        self.server = false;
        self.active = false;
        self.server_connections.clear();
    }

    pub fn set_client_loaded(&mut self, loaded: bool) {
        self.client_loaded = loaded;
        if loaded {
            let queued: Vec<_> = self.packet_queue.drain(..).collect();
            for packet in queued {
                self.handle_client_received(packet);
            }
        } else {
            self.packet_queue.clear();
        }
    }

    pub fn set_client_connected(&mut self) {
        self.active = true;
        self.server = false;
    }

    pub fn mark_server_active(&mut self) {
        self.active = true;
        self.server = true;
    }

    pub fn active(&self) -> bool {
        self.active
    }
    pub fn server(&self) -> bool {
        self.server && self.active
    }
    pub fn client(&self) -> bool {
        !self.server && self.active
    }
    pub fn current_stream(&self) -> Option<&StreamBuilder> {
        self.current_stream.as_ref()
    }
    pub fn queued_len(&self) -> usize {
        self.packet_queue.len()
    }
    pub fn handled_client_packets(&self) -> &[PacketKind] {
        &self.handled_client_packets
    }
    pub fn handled_server_packets(&self) -> &[PacketKind] {
        &self.handled_server_packets
    }

    pub fn handle_client<F>(&mut self, listener: F)
    where
        F: FnMut(&PacketKind) + Send + 'static,
    {
        self.client_listeners.push(Box::new(listener));
    }

    /// Registers the Rust equivalent of Java `net.handleClient(Connect.class, ...)`.
    pub fn handle_client_connect<F>(&mut self, listener: F)
    where
        F: FnMut(&Connect) + Send + 'static,
    {
        self.client_connect_listeners.push(Box::new(listener));
    }

    /// Registers the Rust equivalent of Java `net.handleClient(Disconnect.class, ...)`.
    pub fn handle_client_disconnect<F>(&mut self, listener: F)
    where
        F: FnMut(&Disconnect) + Send + 'static,
    {
        self.client_disconnect_listeners.push(Box::new(listener));
    }

    /// Registers the Rust equivalent of Java `net.handleClient(WorldStream.class, ...)`.
    ///
    /// Mindustry v157.4 only registers `WorldStream` as a streamable packet, so the
    /// reassembled `Streamable` payload is dispatched through this listener.
    pub fn handle_client_world_stream<F>(&mut self, listener: F)
    where
        F: FnMut(&Streamable) + Send + 'static,
    {
        self.client_world_stream_listeners.push(Box::new(listener));
    }

    pub fn handle_client_streamable<F>(&mut self, listener: F)
    where
        F: FnMut(&Streamable) + Send + 'static,
    {
        self.handle_client_world_stream(listener);
    }

    pub fn handle_server<F>(&mut self, listener: F)
    where
        F: FnMut(Option<i32>, &PacketKind) + Send + 'static,
    {
        self.server_listeners.push(Box::new(listener));
    }

    /// Registers the Rust equivalent of Java `net.handleServer(Connect.class, ...)`.
    pub fn handle_server_connect<F>(&mut self, listener: F)
    where
        F: FnMut(Option<i32>, &Connect) + Send + 'static,
    {
        self.server_connect_listeners.push(Box::new(listener));
    }

    /// Registers the Rust equivalent of Java `net.handleServer(Disconnect.class, ...)`.
    pub fn handle_server_disconnect<F>(&mut self, listener: F)
    where
        F: FnMut(Option<i32>, &Disconnect) + Send + 'static,
    {
        self.server_disconnect_listeners.push(Box::new(listener));
    }

    /// Registers the Rust equivalent of Java `net.handleServer(ConnectPacket.class, ...)`.
    pub fn handle_server_connect_packet<F>(&mut self, listener: F)
    where
        F: FnMut(Option<i32>, &ConnectPacket) + Send + 'static,
    {
        self.server_connect_packet_listeners
            .push(Box::new(listener));
    }

    /// Registers the Rust equivalent of Java's high-priority client-to-server
    /// `ConnectConfirmCallPacket` path.
    pub fn handle_server_connect_confirm<F>(&mut self, listener: F)
    where
        F: FnMut(Option<i32>, &ConnectConfirmCallPacket) + Send + 'static,
    {
        self.server_connect_confirm_listeners
            .push(Box::new(listener));
    }

    pub fn connect(
        &mut self,
        ip: &str,
        port: u16,
        success: Box<dyn Fn() + Send + 'static>,
    ) -> io::Result<()> {
        if self.active {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "alreadyconnected",
            ));
        }

        self.provider.connect_client(ip, port, success)?;
        self.active = true;
        self.server = false;
        Ok(())
    }

    pub fn host(&mut self, port: u16) -> io::Result<()> {
        self.provider.host_server(port)?;
        self.active = true;
        self.server = true;
        Ok(())
    }

    pub fn close_server(&mut self) {
        if self.server {
            let kick = PacketKind::KickCallPacket2(KickCallPacket2 {
                reason: KickReason::ServerClose,
            });
            let _ = self.provider.send_server(&kick, true);
        }
        self.clear_client_runtime_state();
        self.provider.close_server();
        self.server = false;
        self.active = false;
        self.server_connections.clear();
    }

    pub fn reset(&mut self) {
        self.close_server();
        self.disconnect();
    }

    pub fn disconnect(&mut self) {
        self.clear_client_runtime_state();
        self.provider.disconnect_client();
        self.server = false;
        self.active = false;
        self.server_connections.clear();
    }

    pub fn discover_servers(&self, callback: HostCallback, done: DoneCallback) {
        self.provider.discover_servers(callback, done);
    }

    pub fn ping_host(&self, address: &str, port: u16, timeout: Duration) -> io::Result<Host> {
        self.provider.ping_host(address, port, timeout)
    }

    pub fn get_connections(&self) -> Vec<NetConnection> {
        self.provider.get_connections()
    }

    pub fn send(&mut self, object: &PacketKind, reliable: bool) -> io::Result<()> {
        if self.server {
            self.provider.send_server(object, reliable)
        } else {
            self.provider.send_client(object, reliable)
        }
    }

    pub fn send_to(
        &mut self,
        connection_id: i32,
        object: &PacketKind,
        reliable: bool,
    ) -> io::Result<()> {
        self.provider
            .send_server_to(connection_id, object, reliable)
    }

    pub fn send_except(
        &mut self,
        except_connection_id: i32,
        object: &PacketKind,
        reliable: bool,
    ) -> io::Result<()> {
        self.provider
            .send_server_except(except_connection_id, object, reliable)
    }

    pub fn set_connect_filter(&mut self, connect_filter: Option<ConnectFilter>) {
        self.provider.set_connect_filter(connect_filter);
    }

    pub fn get_connect_filter(&self) -> Option<ConnectFilter> {
        self.provider.get_connect_filter()
    }

    pub fn dispose(&mut self) {
        self.clear_client_runtime_state();
        self.provider.dispose();
        self.server = false;
        self.active = false;
        self.server_connections.clear();
    }

    fn clear_client_runtime_state(&mut self) {
        self.current_stream = None;
        self.packet_queue.clear();
        self.streams.clear();
        self.client_loaded = false;
    }

    pub fn drain_provider_events(&mut self) -> Vec<ProviderEvent> {
        let events = self.provider.drain_events();
        for event in &events {
            match event {
                ProviderEvent::ClientConnected { address_tcp } => {
                    self.handle_client_received(PacketKind::Connect(Connect {
                        address_tcp: address_tcp.clone(),
                    }));
                }
                ProviderEvent::ClientDisconnected { reason } => {
                    self.handle_client_received(PacketKind::Disconnect(Disconnect {
                        reason: reason.clone(),
                    }));
                    self.active = false;
                }
                ProviderEvent::ClientPacket(packet) => {
                    self.handle_client_received(packet.clone());
                }
                ProviderEvent::ServerConnected {
                    connection_id,
                    address,
                } => {
                    let mut connection = NetConnection::new(address.clone());
                    connection.has_connected = true;
                    self.server_connections
                        .insert(*connection_id, connection.clone());
                    self.handle_server_received_from_connection(
                        Some(*connection_id),
                        connection.has_connected,
                        PacketKind::Connect(Connect {
                            address_tcp: address.clone(),
                        }),
                    );
                }
                ProviderEvent::ServerDisconnected {
                    connection_id,
                    reason,
                } => {
                    let has_connected = self
                        .server_connections
                        .get(connection_id)
                        .map(|connection| connection.has_connected)
                        .unwrap_or(true);
                    self.handle_server_received_from_connection(
                        Some(*connection_id),
                        has_connected,
                        PacketKind::Disconnect(Disconnect {
                            reason: reason.clone(),
                        }),
                    );
                    self.server_connections.remove(connection_id);
                }
                ProviderEvent::ServerPacket {
                    connection_id,
                    packet,
                } => {
                    let has_connected = self
                        .server_connections
                        .get(connection_id)
                        .map(|connection| connection.has_connected)
                        .unwrap_or(false);
                    self.handle_server_received_from_connection(
                        Some(*connection_id),
                        has_connected,
                        packet.clone(),
                    );
                }
            }
        }
        events
    }

    pub fn handle_client_received(&mut self, packet: PacketKind) {
        if !packet.allow(false) {
            return;
        }

        if matches!(packet, PacketKind::Disconnect(_)) {
            self.clear_client_runtime_state();
        }

        match packet {
            PacketKind::StreamBegin(begin) => {
                let builder = StreamBuilder::new(begin.id, begin.packet_type, begin.total);
                self.current_stream = Some(builder.clone());
                self.streams.insert(begin.id, builder);
            }
            PacketKind::StreamChunk(chunk) => {
                let mut completed = None;
                if let Some(builder) = self.streams.get_mut(&chunk.id) {
                    builder.add(&chunk.data);
                    self.current_stream = Some(builder.clone());
                    if builder.is_done() {
                        completed = Some(builder.clone());
                    }
                } else {
                    panic!("Received stream chunk without a StreamBegin beforehand!");
                }
                if let Some(builder) = completed {
                    self.streams.remove(&builder.id);
                    self.current_stream = None;
                    self.handle_client_received(PacketKind::Streamable(builder.build()));
                }
            }
            packet => {
                let priority = packet.priority();
                if self.client_loaded || priority == 2 {
                    self.dispatch_client_packet(packet);
                } else if priority != 0 {
                    self.packet_queue.push_back(packet);
                }
            }
        }
    }

    pub fn handle_server_received(&mut self, connection_has_connected: bool, packet: PacketKind) {
        self.handle_server_received_from_connection(None, connection_has_connected, packet);
    }

    pub fn handle_server_received_from_connection(
        &mut self,
        connection_id: Option<i32>,
        connection_has_connected: bool,
        packet: PacketKind,
    ) {
        if !packet.allow(true) {
            return;
        }
        if connection_has_connected || packet.priority() == 2 {
            self.dispatch_server_packet(connection_id, packet);
        }
    }

    fn dispatch_client_packet(&mut self, packet: PacketKind) {
        self.dispatch_typed_client_packet(&packet);
        for listener in &mut self.client_listeners {
            listener(&packet);
        }
        self.handled_client_packets.push(packet);
    }

    fn dispatch_server_packet(&mut self, connection_id: Option<i32>, packet: PacketKind) {
        self.dispatch_typed_server_packet(connection_id, &packet);
        for listener in &mut self.server_listeners {
            listener(connection_id, &packet);
        }
        self.handled_server_packets.push(packet);
    }

    fn dispatch_typed_client_packet(&mut self, packet: &PacketKind) {
        match packet {
            PacketKind::Connect(connect) => {
                for listener in &mut self.client_connect_listeners {
                    listener(connect);
                }
            }
            PacketKind::Disconnect(disconnect) => {
                for listener in &mut self.client_disconnect_listeners {
                    listener(disconnect);
                }
            }
            PacketKind::Streamable(stream) => {
                for listener in &mut self.client_world_stream_listeners {
                    listener(stream);
                }
            }
            _ => {}
        }
    }

    fn dispatch_typed_server_packet(&mut self, connection_id: Option<i32>, packet: &PacketKind) {
        match packet {
            PacketKind::Connect(connect) => {
                for listener in &mut self.server_connect_listeners {
                    listener(connection_id, connect);
                }
            }
            PacketKind::Disconnect(disconnect) => {
                for listener in &mut self.server_disconnect_listeners {
                    listener(connection_id, disconnect);
                }
            }
            PacketKind::ConnectPacket(connect_packet) => {
                for listener in &mut self.server_connect_packet_listeners {
                    listener(connection_id, connect_packet);
                }
            }
            PacketKind::ConnectConfirmCallPacket(connect_confirm) => {
                for listener in &mut self.server_connect_confirm_listeners {
                    listener(connection_id, connect_confirm);
                }
            }
            PacketKind::PingCallPacket(ping) => {
                if let Some(connection_id) = connection_id {
                    let response = PacketKind::PingResponseCallPacket(PingResponseCallPacket {
                        time: ping.time,
                    });
                    let _ = self.send_to(connection_id, &response, true);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;

    use crate::mindustry::net::packets::packet_ids;

    use super::*;

    #[test]
    fn normal_client_packet_queues_until_loaded() {
        let mut net = Net::default();
        net.handle_client_received(PacketKind::Other {
            id: 9,
            priority: 1,
            allow_client: true,
            allow_server: true,
        });
        assert_eq!(net.queued_len(), 1);
        assert!(net.handled_client_packets().is_empty());
        net.set_client_loaded(true);
        assert_eq!(net.queued_len(), 0);
        assert_eq!(net.handled_client_packets().len(), 1);
    }

    #[test]
    fn client_stream_reassembles_to_streamable() {
        let mut net = Net::default();
        net.handle_client_received(PacketKind::StreamBegin(StreamBegin {
            id: 1,
            total: 3,
            packet_type: 2,
        }));
        net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
            id: 1,
            data: vec![1, 2],
        }));
        assert!(net.handled_client_packets().is_empty());
        net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
            id: 1,
            data: vec![3],
        }));
        assert_eq!(net.handled_client_packets().len(), 1);
        match &net.handled_client_packets()[0] {
            PacketKind::Streamable(stream) => assert_eq!(stream.stream, vec![1, 2, 3]),
            other => panic!("unexpected packet: {other:?}"),
        }
    }

    #[test]
    fn provider_events_flow_into_net_handlers() {
        let provider = EventProvider {
            events: VecDeque::from([
                ProviderEvent::ClientConnected {
                    address_tcp: "127.0.0.1:6567".into(),
                },
                ProviderEvent::ClientPacket(PacketKind::Other {
                    id: 7,
                    priority: 2,
                    allow_client: true,
                    allow_server: true,
                }),
                ProviderEvent::ServerConnected {
                    connection_id: 3,
                    address: "127.0.0.1".into(),
                },
                ProviderEvent::ServerPacket {
                    connection_id: 3,
                    packet: PacketKind::Other {
                        id: 8,
                        priority: 2,
                        allow_client: true,
                        allow_server: true,
                    },
                },
            ]),
        };
        let mut net = Net::new(Box::new(provider));

        let drained = net.drain_provider_events();

        assert_eq!(drained.len(), 4);
        assert!(matches!(
            net.handled_client_packets()[0],
            PacketKind::Connect(_)
        ));
        assert!(matches!(
            net.handled_client_packets()[1],
            PacketKind::Other { id: 7, .. }
        ));
        assert!(matches!(
            net.handled_server_packets()[0],
            PacketKind::Connect(_)
        ));
        assert!(matches!(
            net.handled_server_packets()[1],
            PacketKind::Other { id: 8, .. }
        ));
    }

    #[test]
    fn listeners_receive_dispatched_client_and_server_packets() {
        let client_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let server_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut net = Net::default();

        let client_seen_listener = client_seen.clone();
        net.handle_client(move |packet| {
            if let PacketKind::Other { id, .. } = packet {
                client_seen_listener.lock().unwrap().push(*id);
            }
        });

        net.handle_client_received(PacketKind::Other {
            id: 21,
            priority: 1,
            allow_client: true,
            allow_server: true,
        });
        assert!(client_seen.lock().unwrap().is_empty());

        net.set_client_loaded(true);
        assert_eq!(*client_seen.lock().unwrap(), vec![21]);

        let server_seen_listener = server_seen.clone();
        net.handle_server(move |connection_id, packet| {
            if let PacketKind::Other { id, .. } = packet {
                server_seen_listener
                    .lock()
                    .unwrap()
                    .push((connection_id, *id));
            }
        });

        net.handle_server_received_from_connection(
            Some(42),
            true,
            PacketKind::Other {
                id: 22,
                priority: 1,
                allow_client: true,
                allow_server: true,
            },
        );

        assert_eq!(*server_seen.lock().unwrap(), vec![(Some(42), 22)]);
    }

    #[test]
    fn typed_listeners_receive_core_connectivity_packets_before_generic_listeners() {
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut net = Net::default();

        let client_log = log.clone();
        net.handle_client_connect(move |_| {
            client_log.lock().unwrap().push("client-connect".into());
        });
        let client_log = log.clone();
        net.handle_client_disconnect(move |_| {
            client_log.lock().unwrap().push("client-disconnect".into());
        });
        let client_log = log.clone();
        net.handle_client_world_stream(move |stream| {
            client_log
                .lock()
                .unwrap()
                .push(format!("client-world-stream:{}", stream.stream.len()));
        });
        let client_log = log.clone();
        net.handle_client(move |packet| {
            let label = match packet {
                PacketKind::Connect(_) => "generic-client-connect".to_string(),
                PacketKind::Disconnect(_) => "generic-client-disconnect".to_string(),
                PacketKind::Streamable(stream) => {
                    format!("generic-client-stream:{}", stream.stream.len())
                }
                _ => return,
            };
            client_log.lock().unwrap().push(label);
        });

        net.handle_client_received(PacketKind::Connect(Connect {
            address_tcp: "127.0.0.1:6567".into(),
        }));
        net.handle_client_received(PacketKind::Disconnect(Disconnect {
            reason: "closed".into(),
        }));
        net.handle_client_received(PacketKind::StreamBegin(StreamBegin {
            id: 5,
            total: 3,
            packet_type: packet_ids::WORLD_STREAM,
        }));
        net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
            id: 5,
            data: vec![1, 2],
        }));
        net.handle_client_received(PacketKind::StreamChunk(StreamChunk {
            id: 5,
            data: vec![3],
        }));

        assert_eq!(
            *log.lock().unwrap(),
            vec![
                "client-connect",
                "generic-client-connect",
                "client-disconnect",
                "generic-client-disconnect",
                "client-world-stream:3",
                "generic-client-stream:3",
            ]
        );

        let server_log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut net = Net::default();

        let server_log_clone = server_log.clone();
        net.handle_server_connect(move |connection_id, _| {
            server_log_clone
                .lock()
                .unwrap()
                .push(format!("server-connect:{connection_id:?}"));
        });
        let server_log_clone = server_log.clone();
        net.handle_server_disconnect(move |connection_id, _| {
            server_log_clone
                .lock()
                .unwrap()
                .push(format!("server-disconnect:{connection_id:?}"));
        });
        let server_log_clone = server_log.clone();
        net.handle_server_connect_packet(move |connection_id, packet| {
            server_log_clone.lock().unwrap().push(format!(
                "server-connect-packet:{connection_id:?}:{}",
                packet.name
            ));
        });
        let server_log_clone = server_log.clone();
        net.handle_server_connect_confirm(move |connection_id, _| {
            server_log_clone
                .lock()
                .unwrap()
                .push(format!("server-connect-confirm:{connection_id:?}"));
        });
        let server_log_clone = server_log.clone();
        net.handle_server(move |connection_id, packet| {
            let label = match packet {
                PacketKind::Connect(_) => format!("generic-server-connect:{connection_id:?}"),
                PacketKind::Disconnect(_) => {
                    format!("generic-server-disconnect:{connection_id:?}")
                }
                PacketKind::ConnectPacket(packet) => format!(
                    "generic-server-connect-packet:{connection_id:?}:{}",
                    packet.name
                ),
                PacketKind::ConnectConfirmCallPacket(_) => {
                    format!("generic-server-connect-confirm:{connection_id:?}")
                }
                _ => return,
            };
            server_log_clone.lock().unwrap().push(label);
        });

        net.handle_server_received_from_connection(
            Some(9),
            false,
            PacketKind::Connect(Connect {
                address_tcp: "10.0.0.2:6567".into(),
            }),
        );
        net.handle_server_received_from_connection(
            Some(9),
            false,
            PacketKind::ConnectPacket(ConnectPacket {
                version: 157,
                version_type: "official".into(),
                mods: vec!["mod-a".into()],
                name: "player".into(),
                locale: "en_US".into(),
                uuid: base64::engine::general_purpose::STANDARD.encode([1, 2, 3, 4, 5, 6, 7, 8]),
                usid: "usid".into(),
                mobile: false,
                color: 12,
                uuid_crc32: None,
            }),
        );
        net.handle_server_received_from_connection(
            Some(9),
            false,
            PacketKind::ConnectConfirmCallPacket(ConnectConfirmCallPacket),
        );
        net.handle_server_received_from_connection(
            Some(9),
            true,
            PacketKind::Disconnect(Disconnect {
                reason: "left".into(),
            }),
        );

        assert_eq!(
            *server_log.lock().unwrap(),
            vec![
                "server-connect:Some(9)",
                "generic-server-connect:Some(9)",
                "server-connect-packet:Some(9):player",
                "generic-server-connect-packet:Some(9):player",
                "server-connect-confirm:Some(9)",
                "generic-server-connect-confirm:Some(9)",
                "server-disconnect:Some(9)",
                "generic-server-disconnect:Some(9)",
            ]
        );
    }

    #[test]
    fn close_server_sends_server_close_kick_before_disposing_provider() {
        let sent = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let closed = std::sync::Arc::new(std::sync::Mutex::new(false));
        let provider = RecordingProvider {
            sent: sent.clone(),
            closed: closed.clone(),
        };
        let mut net = Net::new(Box::new(provider));
        net.server = true;
        net.active = true;

        net.close_server();

        let sent = sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        match &sent[0].0 {
            PacketKind::KickCallPacket2(packet) => {
                assert_eq!(packet.reason, KickReason::ServerClose);
            }
            other => panic!("unexpected packet: {other:?}"),
        }
        assert!(sent[0].1);
        assert!(*closed.lock().unwrap());
        assert!(!net.server);
        assert!(!net.active);
    }

    #[test]
    fn disconnect_clears_client_runtime_state() {
        let mut net = Net::default();
        net.current_stream = Some(StreamBuilder::new(7, 2, 3));
        net.packet_queue.push_back(PacketKind::Other {
            id: 1,
            priority: 1,
            allow_client: true,
            allow_server: true,
        });
        net.streams.insert(7, StreamBuilder::new(7, 2, 3));
        net.client_loaded = true;
        net.active = true;

        net.disconnect();

        assert!(net.current_stream.is_none());
        assert!(net.packet_queue.is_empty());
        assert!(net.streams.is_empty());
        assert!(!net.client_loaded);
        assert!(!net.active);
    }

    #[derive(Default)]
    struct EventProvider {
        events: VecDeque<ProviderEvent>,
    }

    impl NetProvider for EventProvider {
        fn connect_client(
            &mut self,
            _ip: &str,
            _port: u16,
            _success: Box<dyn Fn() + Send + 'static>,
        ) -> io::Result<()> {
            Ok(())
        }

        fn send_client(&mut self, _object: &PacketKind, _reliable: bool) -> io::Result<()> {
            Ok(())
        }

        fn disconnect_client(&mut self) {}

        fn discover_servers(&self, _callback: HostCallback, done: DoneCallback) {
            done();
        }

        fn ping_host(&self, _address: &str, _port: u16, _timeout: Duration) -> io::Result<Host> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "not implemented",
            ))
        }

        fn host_server(&mut self, _port: u16) -> io::Result<()> {
            Ok(())
        }

        fn get_connections(&self) -> Vec<NetConnection> {
            Vec::new()
        }

        fn close_server(&mut self) {}

        fn drain_events(&mut self) -> Vec<ProviderEvent> {
            self.events.drain(..).collect()
        }
    }

    #[derive(Clone)]
    struct RecordingProvider {
        sent: std::sync::Arc<std::sync::Mutex<Vec<(PacketKind, bool)>>>,
        closed: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    impl NetProvider for RecordingProvider {
        fn connect_client(
            &mut self,
            _ip: &str,
            _port: u16,
            _success: Box<dyn Fn() + Send + 'static>,
        ) -> io::Result<()> {
            Ok(())
        }

        fn send_client(&mut self, _object: &PacketKind, _reliable: bool) -> io::Result<()> {
            Ok(())
        }

        fn send_server(&mut self, object: &PacketKind, reliable: bool) -> io::Result<()> {
            self.sent.lock().unwrap().push((object.clone(), reliable));
            Ok(())
        }

        fn disconnect_client(&mut self) {}

        fn discover_servers(&self, _callback: HostCallback, done: DoneCallback) {
            done();
        }

        fn ping_host(&self, _address: &str, _port: u16, _timeout: Duration) -> io::Result<Host> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "not implemented",
            ))
        }

        fn host_server(&mut self, _port: u16) -> io::Result<()> {
            Ok(())
        }

        fn get_connections(&self) -> Vec<NetConnection> {
            Vec::new()
        }

        fn close_server(&mut self) {
            *self.closed.lock().unwrap() = true;
        }
    }
}
