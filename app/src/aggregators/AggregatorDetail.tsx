import {
  Await,
  useFetcher,
  useLoaderData,
  useNavigation,
  useParams,
} from "react-router-dom";
import { Aggregator } from "../ApiClient";
import { AccountBreadcrumbs } from "../util";
import { LinkContainer } from "react-router-bootstrap";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import React, { Suspense, useEffect, useState } from "react";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import {
  ArrowRepeat,
  CloudUpload,
  Pencil,
  PencilSquare,
  Trash,
} from "react-bootstrap-icons";
import Table from "react-bootstrap/Table";
import D from "../logo/color/svg/small.svg";
import Placeholder from "react-bootstrap/Placeholder";
import {
  Button,
  ButtonGroup,
  FormControl,
  FormGroup,
  FormLabel,
  Modal,
} from "react-bootstrap";

function Breadcrumbs() {
  let { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };
  let { account_id } = useParams();

  return (
    <AccountBreadcrumbs>
      <LinkContainer to={`/accounts/${account_id}/aggregators`}>
        <Breadcrumb.Item>Aggregators</Breadcrumb.Item>
      </LinkContainer>
      <Breadcrumb.Item active>
        <React.Suspense fallback={<Placeholder animation="glow" xs={6} />}>
          <Await resolve={aggregator}>{(aggregator) => aggregator.name}</Await>
        </React.Suspense>
      </Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

export default function AggregatorDetail() {
  let { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };

  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <Suspense
              fallback={
                <>
                  <CloudUpload /> <Placeholder animation="glow" xs={6} />
                </>
              }
            >
              <Await resolve={aggregator}>
                {(aggregator) => (
                  <>
                    {aggregator.is_first_party ? (
                      <img
                        src={D}
                        style={{ height: "1em", marginTop: "-0.2em" }}
                      />
                    ) : (
                      <CloudUpload />
                    )}{" "}
                    {aggregator.name}
                  </>
                )}
              </Await>
            </Suspense>
          </h1>
        </Col>
      </Row>

      <Row>
        <Col>
          <AggregatorPropertyTable />
          <ButtonGroup>
            <RenameAggregatorButton />
            <RotateBearerTokenButton />
            <DeleteAggregatorButton />
          </ButtonGroup>
        </Col>
      </Row>
    </>
  );
}

function AggregatorPropertyTable() {
  return (
    <Table striped bordered className="overflow-auto" responsive>
      <tbody>
        <TableRow label="Name" value="name" />
        <TableRow label="DAP url" value="dap_url" />
        <TableRow label="API url" value="api_url" />
        <TableRow label="Supported roles" value="role" />
        <TableRow
          label="Supported VDAFs"
          value={({ vdafs }) =>
            vdafs.map((v) => v.replace(/^Prio3/, "").toLowerCase()).join(", ")
          }
        />
        <TableRow
          label="Supported query types"
          value={({ query_types }) =>
            query_types
              .map((v) => v.replaceAll(/([A-Z])/g, " $1").toLowerCase())
              .join(", ")
          }
        />
      </tbody>
    </Table>
  );
}

function TableRow({
  label,
  value,
}: {
  label: React.ReactNode;
  value: keyof Aggregator | ((data: Aggregator) => React.ReactNode);
}) {
  return (
    <tr>
      <td>{label}</td>
      <td>
        <WithAggregator>
          {typeof value === "string"
            ? (aggregator) => aggregator[value]
            : value}
        </WithAggregator>
      </td>
    </tr>
  );
}

export function WithAggregator({
  children,
}: {
  children: (data: Awaited<Aggregator>) => React.ReactNode;
}) {
  let { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };

  return (
    <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
      <Await resolve={aggregator} children={children} />
    </Suspense>
  );
}

function RenameAggregatorButton() {
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

function RotateBearerTokenButton() {
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
        <ArrowRepeat /> Rotate Token
      </Button>
      <Modal show={show} onHide={close}>
        <fetcher.Form method="PATCH">
          <Modal.Header closeButton>
            <Modal.Title>
              Rotate Bearer Token for{" "}
              <WithAggregator>{({ name }) => `"${name}"`}</WithAggregator>
            </Modal.Title>
          </Modal.Header>
          <Modal.Body>
            <FormGroup controlId="bearer_token">
              <FormLabel>New Bearer Token</FormLabel>
              <FormControl name="bearer_token" type="text" />
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
              <ArrowRepeat /> Rotate
            </Button>
          </Modal.Footer>
        </fetcher.Form>
      </Modal>
    </>
  );
}

function DeleteAggregatorButton() {
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
