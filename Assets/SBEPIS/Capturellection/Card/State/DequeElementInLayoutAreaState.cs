using System;
using SBEPIS.Utils;
using UnityEngine;

namespace SBEPIS.Capturellection.CardState
{
	public class DequeElementInLayoutAreaState : StateMachineBehaviour<DequeElementStateMachine>
	{
		protected override void OnEnter()
		{
			if (!State.LayoutAdder)
				throw new NullReferenceException($"DequeStorable {State.Card} doesn't have a LayoutAdder but reached its state");
			DiajectorCaptureLayout layout = State.LayoutAdder.PopAllLayouts();
			layout.inventory.Flush(State.LayoutAdder.Card);
			CardTarget target = layout.AddPermanentTargetAndCard(State.LayoutAdder.Card);
			State.Card.Animator.TeleportTo(target.LerpTarget);
		}
	}
}