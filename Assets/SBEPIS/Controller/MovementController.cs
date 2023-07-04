using KBCore.Refs;
using SBEPIS.Physics;
using UnityEngine;
using CallbackContext = UnityEngine.InputSystem.InputAction.CallbackContext;

namespace SBEPIS.Controller
{
	[RequireComponent(typeof(Rigidbody), typeof(Orientation))]
	public class MovementController : ValidatedMonoBehaviour
	{
		[SerializeField, Self] private new Rigidbody rigidbody;
		[SerializeField, Self] private Orientation orientation;
		
		[SerializeField, Anywhere] private Transform moveAimer;
		
		[SerializeField, Anywhere] private FootBall footBall;
		
		[SerializeField] private float maxGroundSpeed = 8;
		[SerializeField] private float groundAcceleration = 10;
		[SerializeField] private float airAcceleration = 1;
		[SerializeField] private float sprintFactor = 2;
		[SerializeField] private float sprintControlThreshold = 0.9f;
		
		private Vector3 controlsTarget;
		private bool isTryingToSprint;
		private bool isSprinting;
		
		private void FixedUpdate()
		{
			MoveTick();
		}
		
		private void MoveTick()
		{
			UpdateSprinting();
			Accelerate(orientation.RelativeVelocity, orientation.UpDirection);
		}
		
		private void UpdateSprinting()
		{
			if (isSprinting && controlsTarget.magnitude < sprintControlThreshold)
				isSprinting = false;
			else if (!isSprinting && isTryingToSprint && controlsTarget.magnitude > sprintControlThreshold)
				isSprinting = true;
		}
		
		private void Accelerate(Vector3 currentVelocity, Vector3 upDirection)
		{
			Vector3 accelerationControl = moveAimer.right * controlsTarget.x + Vector3.Cross(moveAimer.right, upDirection) * controlsTarget.z;
			AccelerateGround(upDirection, accelerationControl);
			if (accelerationControl != Vector3.zero && !orientation.IsGrounded)
				AccelerateAir(currentVelocity, accelerationControl);
		}
		
		private void AccelerateGround(Vector3 upDirection, Vector3 accelerationControl)
		{
			float maxSpeed = maxGroundSpeed * accelerationControl.magnitude * (isSprinting ? sprintFactor : 1);
			Vector3 newVelocity = accelerationControl.normalized * maxSpeed;
			footBall.Move(upDirection, newVelocity);
		}
		
		private void AccelerateAir(Vector3 currentVelocity, Vector3 accelerationControl)
		{
			//float maxSpeed = Mathf.Max(currentVelocity.magnitude, maxGroundSpeed * accelerationControl.magnitude * (isSprinting ? sprintFactor : 1));
			//Vector3 newVelocity = currentVelocity + Time.fixedDeltaTime * airAcceleration * accelerationControl.normalized;
			//Vector3 clampedNewVelocity = Vector3.ClampMagnitude(newVelocity, maxSpeed);
			//rigidbody.velocity += clampedNewVelocity - currentVelocity;
			
			Vector3 newVelocity = currentVelocity + Time.fixedDeltaTime * airAcceleration * accelerationControl;
			rigidbody.velocity += newVelocity - currentVelocity;
		}
		
		public void OnMove(CallbackContext context)
		{
			Vector2 controls = context.ReadValue<Vector2>();
			controlsTarget = new Vector3(controls.x, 0, controls.y);
		}
		
		public void OnSprint(CallbackContext context)
		{
			isTryingToSprint = context.performed;
		}
	}
}
