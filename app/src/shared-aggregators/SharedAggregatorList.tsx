import {
  Button,
  ButtonGroup,
  Col,
  FormControl,
  FormGroup,
  FormLabel,
  ListGroup,
  ListGroupItem,
  Modal,
  Placeholder,
  Row,
} from "react-bootstrap";
import {
  Await,
  useFetcher,
  useLoaderData,
  useNavigation,
} from "react-router-dom";
import { Aggregator } from "../ApiClient";
import "@github/relative-time-element";
import { Suspense, useEffect, useState } from "react";
import SharedAggregatorForm from "./SharedAggregatorForm";
import { Pencil, PencilSquare, Trash } from "react-bootstrap-icons";
import React from "react";

export const Component = JobQueue;

export function JobQueue() {
  const { aggregators } = useLoaderData() as {
    aggregators: Promise<Aggregator[]>;
  };

  return (
    <>
      <Row style={{ height: "calc(100vh - 56px)" }}>
        <Col sm={6} style={{ overflowY: "auto", height: "100%" }}>
          <ListGroup className="list-group-flush">
            <Suspense fallback={<Placeholder animation="wave" xs={6} />}>
              <Await resolve={aggregators}>
                {(aggregators: Aggregator[]) =>
                  aggregators.length === 0 ? (
                    <ListGroupItem disabled>none</ListGroupItem>
                  ) : (
                    aggregators.map((aggregator) => (
                      <AggregatorRow
                        aggregator={aggregator}
                        key={aggregator.id}
                      />
                    ))
                  )
                }
              </Await>
            </Suspense>
          </ListGroup>
        </Col>
        <Col sm={6}>
          <SharedAggregatorForm />
        </Col>
      </Row>
    </>
  );
}

function AggregatorRow({ aggregator }: { aggregator: Aggregator }) {
  const aggForDisplay = { ...aggregator } as Partial<Aggregator>;
  delete aggForDisplay.account_id;
  delete aggForDisplay.name;
  return (
    <ListGroupItem>
      <h3>{aggregator.name}</h3>
      <pre>
        <code>
          {JSON.stringify(aggForDisplay, null, 2)
            .replaceAll(/"|,|\{|\}| {2}/g, "")
            .trim()}
        </code>
      </pre>
      <ButtonGroup>
        <RenameAggregatorButton aggregator={aggregator} />
        <DeleteAggregatorButton aggregator={aggregator} />
      </ButtonGroup>
    </ListGroupItem>
  );
}

function RenameAggregatorButton({ aggregator }: { aggregator: Aggregator }) {
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
        <PencilSquare />
      </Button>
      <Modal show={show} onHide={close}>
        <fetcher.Form method="PATCH" action={aggregator.id}>
          <Modal.Header closeButton>
            <Modal.Title>Edit "{aggregator.name}"</Modal.Title>
          </Modal.Header>
          <Modal.Body>
            <FormGroup controlId="name">
              <FormLabel>Name</FormLabel>
              <FormControl
                name="name"
                type="text"
                data-1p-ignore
                defaultValue={aggregator.name}
              />
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

function DeleteAggregatorButton({ aggregator }: { aggregator: Aggregator }) {
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
        <Trash />
      </Button>
      <Modal show={show} onHide={close}>
        <Modal.Header closeButton>
          <Modal.Title>Delete "{aggregator.name}"?</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          This aggregator will immediately be removed from the interface and no
          new tasks can be created with it.
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <fetcher.Form method="delete" action={aggregator.id}>
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
