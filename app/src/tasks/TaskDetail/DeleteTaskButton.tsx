import { useFetcher, useNavigation } from "react-router";
import React, { useEffect, useState } from "react";
import { Trash } from "react-bootstrap-icons";
import { Button, Modal } from "react-bootstrap";
import { WithTask } from ".";

export default function DeleteTaskButton() {
  const navigation = useNavigation();

  const [show, setShow] = useState(false);
  const close = React.useCallback(() => setShow(false), []);
  const open = React.useCallback(() => setShow(true), []);
  const fetcher = useFetcher();

  useEffect(() => {
    if (fetcher.data) close();
  }, [fetcher, close]);

  return (
    <>
      <Button variant="danger" size="lg" onClick={open}>
        <Trash /> Delete
      </Button>
      <Modal show={show} onHide={close}>
        <Modal.Header closeButton>
          <Modal.Title>
            Delete <WithTask>{({ name }) => `"${name}"`}</WithTask>?
          </Modal.Title>
        </Modal.Header>
        <Modal.Body>
          This task will be immediately removed from the interface, and
          aggregators will stop accepting reports. It may take a few minutes for
          aggregators to stop accepting reports.
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <fetcher.Form method="DELETE">
            <Button
              variant="danger"
              type="submit"
              disabled={navigation.state === "submitting"}
            >
              <Trash /> Delete
            </Button>
          </fetcher.Form>
        </Modal.Footer>
      </Modal>
    </>
  );
}
