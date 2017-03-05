use game::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MessageType {
    Empty,
    Welcome,
    Intro,
    Title,
    PressAnyKey,
    YouDied,
    Action(ActionMessageType),
    Name(NameMessageType),
    YouRemember(Option<NameMessageType>),
    Unseen,
    Description(DescriptionMessageType),
    NameDescription(NameMessageType),
    NoDescription,
    Menu(MenuMessageType),
    ChooseDirection,
    EmptyWeaponSlotMessage,
    Front,
    Rear,
    Left,
    Right,
    EmptyWeaponSlot,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum NameMessageType {
    Pistol,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ActionMessageType {
    PlayerOpenDoor,
    PlayerCloseDoor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DescriptionMessageType {
    Player,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MenuMessageType {
    NewGame,
    Continue,
    Quit,
    SaveAndQuit,
    Controls,
    Control(InputEvent, Control),
    UnboundControl(Control),
    ControlBinding(Control),
    NextDelivery,
    Shop,
    Garage,
}
