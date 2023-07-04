using KBCore.Refs;
using SBEPIS.Physics;
using SBEPIS.Utils.VectorLinq;
using UnityEngine;

namespace SBEPIS.Controller
{
	[RequireComponent(typeof(Rigidbody), typeof(GravitySum))]
	public class Orientation : ValidatedMonoBehaviour
	{
		[SerializeField, Self] private new Rigidbody rigidbody;
		[SerializeField, Self] private GravitySum gravityNormalizer;
		
		[SerializeField] private SphereCollider groundCheck;
		[SerializeField] private LayerMask groundCheckMask;
		
		private readonly Collider[] groundedColliders = new Collider[1];
		private float delayTimeLeft;
		
		public bool IsGrounded { get; private set; }
		private Collider groundCollider;
		private Rigidbody GroundRigidbody => groundCollider ? groundCollider.attachedRigidbody : null;
		
		public Vector3 RelativeVelocity => GroundRigidbody ? rigidbody.velocity - GroundRigidbody.velocity : rigidbody.velocity;
		public Vector3 GroundVelocity => Vector3.ProjectOnPlane(RelativeVelocity, gravityNormalizer.UpDirection);
		public Vector3 VerticalVelocity => Vector3.Project(RelativeVelocity, gravityNormalizer.UpDirection);
		public Vector3 UpDirection => gravityNormalizer.UpDirection;
		public bool IsFalling => Vector3.Dot(VerticalVelocity, UpDirection) < 0;
		
		private void FixedUpdate()
		{
			if (delayTimeLeft > 0)
				delayTimeLeft -= Time.fixedDeltaTime;
			IsGrounded = delayTimeLeft <= 0 && UnityEngine.Physics.OverlapSphereNonAlloc(groundCheck.transform.TransformPoint(groundCheck.center), groundCheck.radius * groundCheck.transform.lossyScale.Aggregate(Mathf.Max), groundedColliders, groundCheckMask, QueryTriggerInteraction.Ignore) > 0;
			groundCollider = IsGrounded ? groundedColliders[0] : null;
		}
		
		public void Delay(float time)
		{
			delayTimeLeft = Mathf.Max(delayTimeLeft, time);
		}
	}
}
