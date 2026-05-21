use std::collections::{HashMap, VecDeque};

use super::packets::{
    AnnounceCallPacket, ClearObjectivesCallPacket, ClientBinaryPacketReliableCallPacket,
    ClientBinaryPacketUnreliableCallPacket, ClientPacketReliableCallPacket,
    ClientPacketUnreliableCallPacket, ClientPlanSnapshotCallPacket,
    ClientPlanSnapshotReceivedCallPacket, ClientSnapshotCallPacket, CompleteObjectiveCallPacket,
    ConnectCallPacket, ConnectConfirmCallPacket, ConnectPacket, ConstructFinishCallPacket,
    CopyToClipboardCallPacket, CreateBulletCallPacket, CreateMarkerCallPacket,
    CreateWeatherCallPacket, DebugStatusClientCallPacket, DebugStatusClientUnreliableCallPacket,
    DeconstructFinishCallPacket, DeletePlansCallPacket, DestroyPayloadCallPacket,
    DropItemCallPacket, EffectCallPacket, EffectCallPacket2, EffectReliableCallPacket,
    EntitySnapshotCallPacket, FollowUpMenuCallPacket, GameOverCallPacket, HiddenSnapshotCallPacket,
    HideFollowUpMenuCallPacket, HideHudTextCallPacket, InfoMessageCallPacket, InfoPopupCallPacket,
    InfoPopupCallPacket2, InfoPopupReliableCallPacket, InfoPopupReliableCallPacket2,
    InfoToastCallPacket, KickCallPacket, KickCallPacket2, LabelCallPacket, LabelCallPacket2,
    LabelReliableCallPacket, LabelReliableCallPacket2, LandingPadLandedCallPacket,
    LogicExplosionCallPacket, MenuCallPacket, MenuChooseCallPacket, OpenUriCallPacket,
    PayloadDroppedCallPacket, PickedBuildPayloadCallPacket, PickedUnitPayloadCallPacket,
    PingCallPacket, PingLocationCallPacket, PingResponseCallPacket, PlayerDisconnectCallPacket,
    PlayerSpawnCallPacket, RemoveMarkerCallPacket, RemoveQueueBlockCallPacket,
    RemoveTileCallPacket, RemoveWorldLabelCallPacket, RequestBlockSnapshotCallPacket,
    RequestBuildPayloadCallPacket, RequestDebugStatusCallPacket, RequestDropPayloadCallPacket,
    RequestItemCallPacket, RequestUnitPayloadCallPacket, ResearchedCallPacket,
    RotateBlockCallPacket, SectorCaptureCallPacket, SendChatMessageCallPacket,
    SendMessageCallPacket, SendMessageCallPacket2, ServerBinaryPacketReliableCallPacket,
    ServerBinaryPacketUnreliableCallPacket, ServerPacketReliableCallPacket,
    ServerPacketUnreliableCallPacket, SetCameraPositionCallPacket, SetFlagCallPacket,
    SetFloorCallPacket, SetHudTextCallPacket, SetHudTextReliableCallPacket, SetItemCallPacket,
    SetItemsCallPacket, SetLiquidCallPacket, SetLiquidsCallPacket, SetMapAreaCallPacket,
    SetObjectivesCallPacket, SetOverlayCallPacket, SetPlayerTeamEditorCallPacket,
    SetPositionCallPacket, SetRuleCallPacket, SetRulesCallPacket, SetTeamCallPacket,
    SetTeamsCallPacket, SetTileBlocksCallPacket, SetTileCallPacket, SetTileFloorsCallPacket,
    SetTileItemsCallPacket, SetTileLiquidsCallPacket, SetTileOverlaysCallPacket,
    SetUnitCommandCallPacket, SetUnitStanceCallPacket, SoundAtCallPacket, SoundCallPacket,
    SpawnEffectCallPacket, StateSnapshotCallPacket, StreamBegin, StreamChunk,
    SyncVariableCallPacket, TakeItemsCallPacket, TextInputCallPacket, TextInputCallPacket2,
    TextInputResultCallPacket, TileConfigCallPacket, TileTapCallPacket, TraceInfoCallPacket,
    TransferInventoryCallPacket, TransferItemEffectCallPacket, TransferItemToCallPacket,
    TransferItemToUnitCallPacket, UnitBlockSpawnCallPacket, UnitBuildingControlSelectCallPacket,
    UnitCapDeathCallPacket, UnitClearCallPacket, UnitControlCallPacket, UnitDeathCallPacket,
    UnitDespawnCallPacket, UnitDestroyCallPacket, UnitEnteredPayloadCallPacket,
    UnitEnvDeathCallPacket, UnitSafeDeathCallPacket, UnitSpawnCallPacket,
    UnitTetherBlockSpawnedCallPacket, UpdateGameOverCallPacket, UpdateMarkerCallPacket,
    UpdateMarkerTextCallPacket, UpdateMarkerTextureCallPacket, WarningToastCallPacket,
    WorldDataBeginCallPacket,
};
use super::streamable::{StreamBuilder, Streamable};

#[derive(Debug, Clone, PartialEq)]
pub enum PacketKind {
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
            PacketKind::StreamBegin(_)
            | PacketKind::StreamChunk(_)
            | PacketKind::Streamable(_)
            | PacketKind::ConnectPacket(_)
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
            | PacketKind::PingResponseCallPacket(_)
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

#[derive(Debug, Default)]
pub struct Net {
    server: bool,
    active: bool,
    client_loaded: bool,
    current_stream: Option<StreamBuilder>,
    packet_queue: VecDeque<PacketKind>,
    streams: HashMap<i32, StreamBuilder>,
    handled_client_packets: Vec<PacketKind>,
    handled_server_packets: Vec<PacketKind>,
}

impl Net {
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

    pub fn handle_client_received(&mut self, packet: PacketKind) {
        if !packet.allow(false) {
            return;
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
                    self.handled_client_packets.push(packet);
                } else if priority != 0 {
                    self.packet_queue.push_back(packet);
                }
            }
        }
    }

    pub fn handle_server_received(&mut self, connection_has_connected: bool, packet: PacketKind) {
        if !packet.allow(true) {
            return;
        }
        if connection_has_connected || packet.priority() == 2 {
            self.handled_server_packets.push(packet);
        }
    }
}

#[cfg(test)]
mod tests {
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
}
