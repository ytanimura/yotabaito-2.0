use crate::*;

pub struct BackGround;

impl Component for BackGround {
    type Message = ();
    type Properties = ();

	fn create(_: &Context<Self>) -> Self {
		Self
	}

	fn view(&self, _: &Context<Self>) -> Html {
		html! {
			<div class="background"></div>
		}
	}
}
