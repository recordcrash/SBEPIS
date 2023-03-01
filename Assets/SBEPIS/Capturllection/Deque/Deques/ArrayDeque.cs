using System.Collections;
using System.Collections.Generic;
using System.Linq;
using UnityEngine;

namespace SBEPIS.Capturllection.Deques
{
	public class ArrayDeque : DequeBase
	{
		public float cardDistance = 0.1f;
		public Quaternion cardRotation = Quaternion.identity;
		public float wobbleHeight = 0.1f;
		public float wobbleTimeOffset = 1;
		
		private float time;
		
		public override void Tick(List<DequeStorable> cards, float delta)
		{
			time += delta;
		}
		
		public override void LayoutTargets(List<DequeStorable> cards, Dictionary<DequeStorable, CardTarget> targets)
		{
			int i = 0;
			Vector3 right = cardDistance * (targets.Count - 1) / 2 * Vector3.left;
			foreach ((DequeStorable card, CardTarget target) in InOrder(cards, targets))
			{
				Vector3 up = Mathf.Sin(time + i * wobbleTimeOffset) * wobbleHeight * Vector3.up;
				target.transform.localPosition = right + up;
				target.transform.localRotation = cardRotation;
				right += Vector3.right * cardDistance;
				i++;
			}
		}
		
		public override bool CanFetch(List<DequeStorable> cards, DequeStorable card) => true;
		
		public override int GetIndexToStoreInto(List<DequeStorable> cards) => GetFirstIndexWhere(cards, HasEmptyContainer, OrFirstCard);
		
		public override int GetIndexToInsertCardBetween(List<DequeStorable> cards, DequeStorable card) => GetFirstIndexWhere(cards, HasEmptyContainer, OrAfterAllCards);
	}
}
