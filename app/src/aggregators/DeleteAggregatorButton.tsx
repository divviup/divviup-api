import { useFetcher, useNavigation } from "react-router";
import React, { useEffect, useState } from "react";
import { Trash } from "react-bootstrap-icons";
import { Button, Modal } from "react-bootstrap";
import { WithAggregator } from "./AggregatorDetail";

export default function DeleteAggregatorButton() {
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
      <Button
        variant="outline-danger"
        className="ml-auto"
        size="sm"
        onClick={open}
      >
        <Trash /> Delete
      </Button>
      <Modal show={show} onHide={close}>
        <Modal.Header closeButton>
          <Modal.Title>
            Delete <WithAggregator>{({ name }) => `"${name}"`}</WithAggregator>?
          </Modal.Title>
        </Modal.Header>
        <Modal.Body>
          This aggregator will immediately be removed from the interface and no
          new tasks can be created with it.
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <fetcher.Form method="delete">
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
