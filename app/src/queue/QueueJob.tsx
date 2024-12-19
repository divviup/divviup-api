import React from "react";
import { Col, Table } from "react-bootstrap";
import { CheckSquare, Stopwatch, XCircle } from "react-bootstrap-icons";
import { useLoaderData } from "react-router";
import { QueueJob } from "../ApiClient";
import { DateTime } from "luxon";
import { Link } from "react-router";

export const Component = QueueJobComponent;

export function QueueJobComponent() {
  const job = useLoaderData() as QueueJob;

  return (
    <Col xs="8">
      <Job job={job} key={job.id} />
    </Col>
  );
}

function JobStatus({ job }: { job: QueueJob }) {
  switch (job.status) {
    case "Success":
      return <CheckSquare />;
    case "Failed":
      return <XCircle />;
    case "Pending":
      return <Stopwatch />;
  }
}

function LabeledRow({
  label,
  value,
  children,
}: {
  label: string;
  value?: string;
  children?: React.ReactNode;
}) {
  return (
    <tr>
      <td>{label}</td>
      <td className="overflow-auto">
        <>
          {value}
          {children}
        </>
      </td>
    </tr>
  );
}

function Job({ job }: { job: QueueJob }) {
  return (
    <>
      <h3>
        <JobStatus job={job} /> {job.job.type}
      </h3>

      <Table striped bordered className="overflow-auto" responsive>
        <tbody>
          <LabeledRow label="created">
            {DateTime.fromISO(job.created_at)
              .toLocal()
              .toLocaleString(DateTime.DATETIME_SHORT)}
          </LabeledRow>

          <LabeledRow label="updated">
            {DateTime.fromISO(job.updated_at)
              .toLocal()
              .toLocaleString(DateTime.DATETIME_SHORT)}
          </LabeledRow>

          {job.scheduled_at ? (
            <LabeledRow label="scheduled">
              {DateTime.fromISO(job.scheduled_at)
                .toLocal()
                .toLocaleString(DateTime.DATETIME_SHORT)}
            </LabeledRow>
          ) : null}

          {job.failure_count > 0 ? (
            <LabeledRow label="failures">{job.failure_count}</LabeledRow>
          ) : null}

          <LabeledRow label="job">
            <pre>
              <code>{JSON.stringify(job.job, null, 2)}</code>
            </pre>
          </LabeledRow>

          {job.parent_id ? (
            <LabeledRow label="parent">
              <Link to={`/admin/queue/${job.parent_id}`}>{job.parent_id}</Link>
            </LabeledRow>
          ) : null}

          {job.child_id ? (
            <LabeledRow label="child">
              <Link to={`/admin/queue/${job.child_id}`}>
                <>{job.child_id}</>
              </Link>
            </LabeledRow>
          ) : null}
        </tbody>
      </Table>
    </>
  );
}
