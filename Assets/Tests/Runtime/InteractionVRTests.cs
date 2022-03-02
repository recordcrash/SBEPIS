using NUnit.Framework;
using SBEPIS.Utils;
using System.Collections;
using UnityEngine;
using UnityEngine.TestTools;
using UnityEngine.InputSystem;
using OculusTouchController = Unity.XR.Oculus.Input.OculusTouchController;

namespace SBEPIS.Tests
{
	public class InteractionVRTests : InputTestFixture, IInteractionTest
	{
		private InteractionVRScene scene;

		private OculusTouchController controller;
		private InputAction grabAction;

		public override void Setup()
		{
			base.Setup();

			scene = TestUtils.GetTestingPrefab<InteractionVRScene>();

			controller = InputSystem.AddDevice<OculusTouchController>();
			grabAction = new InputAction("Grab", InputActionType.Button, "<XRController>/gripPressed");
			grabAction.performed += scene.grabber.OnGrab;
			grabAction.canceled += scene.grabber.OnGrab;
			grabAction.Enable();
		}

		public override void TearDown()
		{
			base.TearDown();

			Object.Destroy(scene.gameObject);
		}

		[UnityTest]
		public IEnumerator GrabGrabsGrabbables()
		{
			scene.grabber.transform.position = scene.grabbable.transform.position;
			yield return new WaitForFixedUpdate();

			Press(controller.gripPressed);
			yield return null;

			Assert.AreEqual(scene.grabbable, scene.grabber.heldGrabbable);
		}

		[UnityTest]
		public IEnumerator GrabLiftsGrabbables()
		{
			Vector3 oldPosition = scene.grabbable.transform.position;

			scene.grabber.transform.position = oldPosition;
			yield return new WaitForFixedUpdate();

			Press(controller.gripPressed);
			yield return null;

			scene.grabber.transform.position += Vector3.up;
			Assert.That(scene.grabbable.transform.position.y, Is.LessThanOrEqualTo(oldPosition.y));
			yield return new WaitForFixedUpdate();

			Assert.That(scene.grabbable.transform.position.y, Is.GreaterThan(oldPosition.y));
		}

		[UnityTest]
		public IEnumerator GrabbingSetsLayerToHeldItem()
		{
			scene.grabber.transform.position = scene.grabbable.transform.position;
			yield return new WaitForFixedUpdate();

			Press(controller.gripPressed);
			yield return null;

			Assert.That(scene.grabbable.gameObject.IsOnLayer(LayerMask.GetMask("Held Item")));
		}

		[UnityTest]
		public IEnumerator UngrabbingSetsLayerToDefault()
		{
			scene.grabber.transform.position = scene.grabbable.transform.position;
			yield return new WaitForFixedUpdate();

			Press(controller.gripPressed);
			yield return null;

			Release(controller.gripPressed);
			yield return null;

			Assert.That(scene.grabbable.gameObject.IsOnLayer(LayerMask.GetMask("Default")));
		}
	}
}