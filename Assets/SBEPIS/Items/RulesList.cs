using System.Linq;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using SBEPIS.Bits;

namespace SBEPIS.Items
{
	[CreateAssetMenu(menuName = "Bits/Rules List")]
	public class RulesList : ScriptableObject, IEnumerable<Rule>
	{
		public Rule[] rules = {
			new AeratedAttachRule(),
			new DefaultReplaceRule(),
		};

		public IEnumerator<Rule> GetEnumerator()
		{
			return rules.Cast<Rule>().GetEnumerator();
		}

		IEnumerator IEnumerable.GetEnumerator()
		{
			return GetEnumerator();
		}
	}

	public abstract class Rule
	{
		public abstract bool IsApplicable(BitSet totalBits);
		public abstract void Apply(ItemBase itemBase, ItemBase module);
	}

	public class AeratedAttachRule : Rule
	{
		public override bool IsApplicable(BitSet totalBits) => totalBits.Has(LeastBits.Aerated);

		public override void Apply(ItemBase itemBase, ItemBase module)
		{
			module.transform.Replace(itemBase.aeratedAttachmentPoint);
			itemBase.aeratedAttachmentPoint = module.aeratedAttachmentPoint;
		}
	}

	public class DefaultReplaceRule : Rule
	{
		public override bool IsApplicable(BitSet totalBits) => true;

		public override void Apply(ItemBase itemBase, ItemBase module)
		{
			module.transform.Replace(itemBase.replaceObject);
			itemBase.replaceObject = module.replaceObject;
		}
	}
}