using SBEPIS.Utils;

namespace SBEPIS.Capturllection.CardState
{
	public class CardMoveToLowerDequeState : CardTargettingState
	{
		protected override LerpTarget GetTargetToTargetTo(Card card) => card.Owner.Deque.lowerTarget;
	}
}
