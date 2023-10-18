import { useFetcher, useNavigation } from "react-router-dom";
import React, { useEffect, useState } from "react";
import { Pencil, PencilSquare } from "react-bootstrap-icons";
import {
  Button,
  FormControl,
  FormGroup,
  FormLabel,
  Modal,
} from "react-bootstrap";
import { WithAggregator } from "./AggregatorDetail";

export default function RenameAggregatorButton() {
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
        variant="outline-secondary"
        className="ml-auto"
        size="sm"
        onClick={open}
      >
        <PencilSquare /> Rename
      </Button>
      <Modal show={show} onHide={close}>
        <fetcher.Form method="PATCH">
          <Modal.Header closeButton>
            <Modal.Title>
              Rename{" "}
              <WithAggregator>{({ name }) => `"${name}"`}</WithAggregator>
            </Modal.Title>
          </Modal.Header>
          <Modal.Body>
            <FormGroup controlId="name">
              <FormLabel>Name</FormLabel>
              <WithAggregator>
                {({ name }) => (
                  <FormControl
                    name="name"
                    type="text"
                    data-1p-ignore
                    defaultValue={name}
                  />
                )}
              </WithAggregator>
            </FormGroup>
          </Modal.Body>
          <Modal.Footer>
            <Button variant="secondary" onClick={close}>
              Close
            </Button>
            <Button
              variant="primary"
              type="submit"
              disabled={navigation.state === "submitting"}
            >
              <Pencil /> Edit
            </Button>
          </Modal.Footer>
        </fetcher.Form>
      </Modal>
    </>
  );
}
