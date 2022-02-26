
#[derive(Copy, Clone)]
pub enum WSEventValue {
    Unknown,
    Missing,
    NotFound,
    Disconnect,
    Connect(bool),
    Volume(i32),
    PrevTrack(bool),
    NextTrack(bool),
    TogglePlayPause(bool),
    ToggleShuffle(bool),
    ToggleRepeatState(bool),
}

#[derive(Copy, Clone)]
pub enum MyAppMessage {
    ClickConnect,
    PrevTrack,
    NextTrack,
    PlayPause,
    ToggleShuffle,
    ToggleRepeat,
    ClickPower,
    ChangeVolume,
    WSEventValue(WSEventValue),
}

#[derive(Copy, Clone)]
pub enum PowerOption {
    Shutdown,
    Reboot,
    Unknown,
}
