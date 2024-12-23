import { Await, useLoaderData, useParams } from "react-router";
import { Aggregator } from "../ApiClient";
import { AccountBreadcrumbs } from "../util";
import { LinkContainer } from "react-router-bootstrap";
import Breadcrumb from "react-bootstrap/Breadcrumb";
import React, { Suspense } from "react";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import { CloudUpload, Trash3Fill } from "react-bootstrap-icons";
import Table from "react-bootstrap/Table";
import D from "../logo/color/svg/small.svg";
import Placeholder from "react-bootstrap/Placeholder";
import { ButtonGroup } from "react-bootstrap";
import RotateBearerTokenButton from "./RotateBearerTokenButton";
import RenameAggregatorButton from "./RenameAggregatorButton";
import DeleteAggregatorButton from "./DeleteAggregatorButton";
import Alert from "react-bootstrap/Alert";

function Breadcrumbs() {
  const { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };
  const { accountId } = useParams();

  return (
    <AccountBreadcrumbs>
      <LinkContainer to={`/accounts/${accountId}/aggregators`}>
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

function AggregatorTitle(): React.ReactNode {
  return (
    <h1>
      <WithAggregator
        fallback={
          <>
            <CloudUpload /> <Placeholder animation="glow" xs={6} />
          </>
        }
      >
        {({ is_first_party, name }) => (
          <>
            {is_first_party ? (
              <img src={D} style={{ height: "1em", marginTop: "-0.2em" }} />
            ) : (
              <CloudUpload />
            )}{" "}
            {name}
          </>
        )}
      </WithAggregator>
    </h1>
  );
}

function DeletedAggregatorMessage() {
  return (
    <WithAggregator>
      {({ deleted_at }) =>
        deleted_at ? (
          <Alert variant="warning">
            <h3>
              {" "}
              <Trash3Fill /> This aggregator has been deleted.
            </h3>
            <p>
              You will continue to be able to view it, but new tasks cannot be
              provisioned against it.
            </p>
          </Alert>
        ) : (
          <></>
        )
      }
    </WithAggregator>
  );
}

export default function AggregatorDetail() {
  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <AggregatorTitle />
        </Col>
      </Row>

      <Row>
        <Col>
          <AggregatorPropertyTable />
          <WithAggregator>
            {({ account_id }) =>
              account_id ? (
                <ButtonGroup>
                  <RenameAggregatorButton />
                  <RotateBearerTokenButton />
                  <DeleteAggregatorButton />
                </ButtonGroup>
              ) : (
                <></>
              )
            }
          </WithAggregator>
        </Col>
      </Row>
      <DeletedAggregatorMessage />
    </>
  );
}

function AggregatorPropertyTable() {
  return (
    <Table striped bordered className="overflow-auto" responsive>
      <tbody>
        <TableRow label="Name" value="name" />
        <TableRow label="Protocol" value="protocol" />
        <TableRow label="DAP url" value="dap_url" />
        <TableRow label="API url" value="api_url" />
        <TableRow label="Supported roles" value="role" />
        <TableRow
          label="Supported functions"
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
        <TableRow
          label="Supported features"
          value={({ features }) =>
            features
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
  fallback,
}: {
  children: (data: Awaited<Aggregator>) => React.ReactNode;
  fallback?: React.ReactNode;
}) {
  const { aggregator } = useLoaderData() as {
    aggregator: Promise<Aggregator>;
  };

  return (
    <Suspense fallback={fallback || <Placeholder animation="glow" xs={6} />}>
      <Await resolve={aggregator}>{children}</Await>
    </Suspense>
  );
}
