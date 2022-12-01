use homotopy_core::Direction;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::DiagramGlMessage;
use crate::{
    app::{Icon, IconSize},
    components::delta::State,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayState {
    Playing,
    Paused,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopingBehaviour {
    FillFoward,
    Boomerang,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Properties, PartialEq)]
pub struct ScrubProperties {
    pub slices: usize,
    pub dispatch: Callback<DiagramGlMessage>,
}

pub struct ScrubComponent {
    pub scrub_state: ScrubState,
}

impl Component for ScrubComponent {
    type Message = ScrubAction;
    type Properties = ScrubProperties;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.props()
            .dispatch
            .emit(DiagramGlMessage::ScrubCallback(ctx.link().callback(|x| x)));
        Self {
            scrub_state: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let t = self.scrub_state.t;
        self.scrub_state.update(&msg);
        // Notify directly DiagramGl that time has increased
        if (t - self.scrub_state.t).abs() > f32::EPSILON {
            ctx.props()
                .dispatch
                .emit(DiagramGlMessage::Scrub(self.scrub_state.t));
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        const RANGE: i32 = 1000;

        let play_pause = {
            let state = if self.scrub_state.state == PlayState::Playing {
                PlayState::Paused
            } else {
                PlayState::Playing
            };
            ctx.link().callback(move |_| ScrubAction::SetState(state))
        };
        let toggle_looping = {
            let behaviour = if self.scrub_state.behaviour == LoopingBehaviour::FillFoward {
                LoopingBehaviour::Boomerang
            } else {
                LoopingBehaviour::FillFoward
            };
            ctx.link()
                .callback(move |_| ScrubAction::SetLooping(behaviour))
        };
        let rewind = ctx.link().callback(|_| ScrubAction::Scrub(0.));
        let fast_forward = ctx.link().callback(|_| ScrubAction::Scrub(1.));
        let set_speed = ctx.link().callback(|_| ScrubAction::ChangeSpeed);
        let on_mouse_down = ctx.link().callback(|_| ScrubAction::Push);
        let on_mouse_up = ctx.link().callback(|_| ScrubAction::Pop);
        let scrub = {
            let inner = ctx.link().callback(|x| x);
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                let updated = input.value().parse::<u32>().unwrap_or(0);
                inner.emit(ScrubAction::Scrub(updated as f32 / RANGE as f32));
            })
        };

        let t = f32::round(RANGE as f32 * self.scrub_state.t);
        let play_pause_icon = if self.scrub_state.state == PlayState::Paused {
            "play_arrow"
        } else {
            "pause"
        };
        let looping_icon = if self.scrub_state.behaviour == LoopingBehaviour::FillFoward {
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
                        {self.scrub_state.speed.as_str()}
                    </span>
                </div>
            </div>
        }
    }
}
