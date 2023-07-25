import {
  Button,
  Col,
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
import { Trash } from "react-bootstrap-icons";
import React from "react";

export const Component = JobQueue;

export function JobQueue() {
  let { aggregators } = useLoaderData() as {
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
  let aggWithoutAccountId = { ...aggregator } as Partial<Aggregator>;
  delete aggWithoutAccountId.account_id;
  return (
    <ListGroupItem>
      <pre>
        <code>
          {JSON.stringify(aggWithoutAccountId, null, 2).replaceAll(
            /"|,|\{|\}|  /g,
            ""
          )}
        </code>
      </pre>
      <DeleteAggregatorButton aggregator={aggregator} />
    </ListGroupItem>
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
          <Modal.Title>
            Confirm Aggregator Deletion ({aggregator.name})
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
