use homotopy_core::Direction;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{
    app::{Icon, IconSize},
    components::delta::{Delta, DeltaAgent, State},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayState {
    Playing,
    Paused,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LoopingBehaviour {
    FillFoward,
    Boomerang,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum PlaybackSpeed {
    Half,
    Normal,
    Fast,
    Faster,
}

impl PlaybackSpeed {
    fn next(self) -> Self {
        match self {
            Self::Half => Self::Normal,
            Self::Normal => Self::Fast,
            Self::Fast => Self::Faster,
            Self::Faster => Self::Half,
        }
    }

    fn modifier(self) -> f32 {
        match self {
            Self::Half => 0.5,
            Self::Normal => 1.,
            Self::Fast => 1.5,
            Self::Faster => 2.,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Half => "0.5x",
            Self::Normal => "1x",
            Self::Fast => "1.5x",
            Self::Faster => "2x",
        }
    }
}

pub enum ScrubAction {
    SetState(PlayState),
    Push,
    Pop,
    SetLooping(LoopingBehaviour),
    ChangeSpeed,
    Scrub(f32),
    Advance(f32),
}

#[derive(PartialEq, Clone)]
pub struct ScrubState {
    pub t: f32,
    direction: Direction,
    state: PlayState,
    speed: PlaybackSpeed,
    behaviour: LoopingBehaviour,
    pushed: Option<PlayState>,
}

impl State for ScrubState {
    type Action = ScrubAction;

    fn update(&mut self, action: &Self::Action) {
        match *action {
            ScrubAction::SetState(state) => self.state = state,
            ScrubAction::Push => {
                self.pushed = Some(self.state);
                self.state = PlayState::Paused;
            }
            ScrubAction::Pop => {
                self.state = self.pushed.take().unwrap_or(PlayState::Paused);
            }
            ScrubAction::SetLooping(behaviour) => {
                self.behaviour = behaviour;
                self.direction = Direction::Forward;
            }
            ScrubAction::ChangeSpeed => self.speed = self.speed.next(),
            ScrubAction::Scrub(t) => self.t = t.clamp(0., 1.),
            ScrubAction::Advance(delta) if self.state == PlayState::Playing => {
                let delta = delta * self.speed.modifier();
                match self.behaviour {
                    LoopingBehaviour::FillFoward => {
                        self.t += delta;
                        if self.t > 1. {
                            self.state = PlayState::Paused;
                            self.t = 1.;
                        }
                    }
                    LoopingBehaviour::Boomerang => {
                        if self.direction == Direction::Forward {
                            self.t += delta;
                            if self.t > 1. {
                                self.direction = Direction::Backward;
                                self.t = 1.;
                            }
                        } else {
                            self.t -= delta;
                            if self.t <= 0. {
                                self.direction = Direction::Forward;
                                self.t = 0.;
                            }
                        }
                    }
                }
            }
            ScrubAction::Advance(_) => {}
        }
    }
}

impl Default for ScrubState {
    fn default() -> Self {
        Self {
            t: 0.,
            direction: Direction::Forward,
            state: PlayState::Playing,
            speed: PlaybackSpeed::Normal,
            behaviour: LoopingBehaviour::Boomerang,
            pushed: Default::default(),
        }
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct ScrubProperties {
    pub slices: usize,
}

pub enum ScrubMessage {
    Delta(ScrubState),
}

pub struct ScrubComponent {
    local: ScrubState,
    _delta: Delta<ScrubState>,
}

impl Component for ScrubComponent {
    type Message = ScrubMessage;
    type Properties = ScrubProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let delta = Delta::new();
        let link = ctx.link().clone();
        delta.register(Box::new(move |agent: &DeltaAgent<ScrubState>, _| {
            let state = agent.state().clone();
            link.send_message(ScrubMessage::Delta(state));
        }));

        Self {
            local: ScrubState::default(),
            _delta: delta,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let ScrubMessage::Delta(state) = msg;
        (self.local != state) && {
            self.local = state;
            true
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        const RANGE: i32 = 1000;

        let play_pause = {
            let delta = Delta::<ScrubState>::new();
            let state = if self.local.state == PlayState::Playing {
                PlayState::Paused
            } else {
                PlayState::Playing
            };
            Callback::from(move |_: MouseEvent| {
                delta.emit(ScrubAction::SetState(state));
            })
        };
        let toggle_looping = {
            let delta = Delta::<ScrubState>::new();
            let behaviour = if self.local.behaviour == LoopingBehaviour::FillFoward {
                LoopingBehaviour::Boomerang
            } else {
                LoopingBehaviour::FillFoward
            };
            Callback::from(move |_: MouseEvent| {
                delta.emit(ScrubAction::SetLooping(behaviour));
            })
        };
        let rewind = {
            let delta = Delta::<ScrubState>::new();
            Callback::from(move |_: MouseEvent| {
                delta.emit(ScrubAction::Scrub(0.));
            })
        };
        let fast_forward = {
            let delta = Delta::<ScrubState>::new();
            Callback::from(move |_: MouseEvent| {
                delta.emit(ScrubAction::Scrub(1.));
            })
        };
        let set_speed = {
            let delta = Delta::<ScrubState>::new();
            Callback::from(move |_: MouseEvent| {
                delta.emit(ScrubAction::ChangeSpeed);
            })
        };
        let on_mouse_down = {
            let delta = Delta::<ScrubState>::new();
            Callback::from(move |_: MouseEvent| {
                delta.emit(ScrubAction::Push);
            })
        };
        let on_mouse_up = {
            let delta = Delta::<ScrubState>::new();
            Callback::from(move |_: MouseEvent| {
                delta.emit(ScrubAction::Pop);
            })
        };
        let scrub = {
            let delta = Delta::<ScrubState>::new();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                let updated = input.value().parse::<u32>().unwrap_or(0);
                delta.emit(ScrubAction::Scrub(updated as f32 / RANGE as f32));
            })
        };

        let t = f32::round(RANGE as f32 * self.local.t);
        let play_pause_icon = if self.local.state == PlayState::Paused {
            "play_arrow"
        } else {
            "pause"
        };
        let looping_icon = if self.local.behaviour == LoopingBehaviour::FillFoward {
            "keyboard_tab"
        } else {
            "loop"
        };

        html! {
            <div class="workspace__scrub">
                <div class="workspace__toolbar__segment">
                    <span
                        class="workspace__toolbar__button"
                        onclick={rewind}
                    >
                        <Icon name="skip_previous" size={IconSize::Icon24} />
                    </span>
                    <span
                        class="workspace__toolbar__button"
                        onclick={play_pause}
                    >
                        <Icon name={play_pause_icon} size={IconSize::Icon24} />
                    </span>
                    <span
                        class="workspace__toolbar__button"
                        onclick={fast_forward}
                    >
                        <Icon name="skip_next" size={IconSize::Icon24} />
                    </span>
                    <input
                        type={"range"}
                        min={0}
                        max={RANGE.to_string()}
                        value={t.to_string()}
                        onmousedown={on_mouse_down}
                        oninput={scrub}
                        onmouseup={on_mouse_up}
                    />
                    <span
                        class="workspace__toolbar__button"
                        onclick={toggle_looping}
                    >
                        <Icon name={looping_icon} size={IconSize::Icon24} />
                    </span>
                    <span
                        class="workspace__toolbar__button workspace__scrub__speed"
                        onclick={set_speed}
                    >
                        {self.local.speed.as_str()}
                    </span>
                </div>
            </div>
        }
    }
}
