import Breadcrumb from "react-bootstrap/Breadcrumb";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Button from "react-bootstrap/Button";
import { AccountBreadcrumbs, WithAccount } from "../util";
import { Check, PencilFill, Key, KeyFill, Trash } from "react-bootstrap-icons";
import { Suspense, useCallback, useEffect, useState } from "react";
import {
  Await,
  Form,
  useFetcher,
  useLoaderData,
  useNavigation,
} from "react-router-dom";
import { HpkeConfig } from "../ApiClient";
import Table from "react-bootstrap/Table";
import React from "react";
import { DateTime } from "luxon";
import FormControl from "react-bootstrap/FormControl";
import FormGroup from "react-bootstrap/FormGroup";
import InputGroup from "react-bootstrap/InputGroup";
import Modal from "react-bootstrap/Modal";
import Placeholder from "react-bootstrap/Placeholder";
import HpkeConfigForm from "./HpkeConfigForm";

export default function HpkeConfigs() {
  const navigation = useNavigation();
  return (
    <>
      <Breadcrumbs />
      <Row>
        <Col>
          <h1>
            <Key />{" "}
            <Suspense fallback={<Placeholder animation="glow" xs={6} />}>
              <WithAccount>{(account) => account.name}</WithAccount>
            </Suspense>{" "}
            HPKE Configs
          </h1>
        </Col>
      </Row>
      <Row className="mb-3">
        <Col>
          <HpkeConfigForm />
        </Col>
      </Row>
      <Row>
        <Col>
          <HpkeConfigList />
        </Col>
      </Row>
    </>
  );
}

function Breadcrumbs() {
  return (
    <AccountBreadcrumbs>
      <Breadcrumb.Item active>HPKE Configs</Breadcrumb.Item>
    </AccountBreadcrumbs>
  );
}

function HpkeConfigList() {
  let { hpkeConfigs } = useLoaderData() as {
    hpkeConfigs: Promise<HpkeConfig[]>;
  };

  return (
    <Table>
      <thead>
        <tr>
          <td>Name</td>
          <td>KEM</td>
          <td>KDF</td>
          <td>AEAD</td>
          <td>Created</td>
          <td></td>
        </tr>
      </thead>
      <tbody>
        <Suspense>
          <Await resolve={hpkeConfigs}>
            {(hpkeConfigs: HpkeConfig[]) =>
              hpkeConfigs.map((hpkeConfig) => (
                <HpkeConfigRow key={hpkeConfig.id} hpkeConfig={hpkeConfig} />
              ))
            }
          </Await>
        </Suspense>
      </tbody>
    </Table>
  );
}

function Name({ hpkeConfig }: { hpkeConfig: HpkeConfig }) {
  let [isEditing, setEditing] = useState(false);
  let edit = useCallback(() => setEditing(true), [setEditing]);
  let fetcher = useFetcher();
  useEffect(() => {
    if (fetcher.data) setEditing(false);
  }, [fetcher, setEditing]);
  if (isEditing) {
    return (
      <fetcher.Form action={hpkeConfig.id} method="patch">
        <FormGroup>
          <InputGroup>
            <FormControl
              type="text"
              name="name"
              defaultValue={hpkeConfig.name || ""}
              data-1p-ignore
              autoFocus
            />
            <Button type="submit">
              <Check />
            </Button>
          </InputGroup>
        </FormGroup>
      </fetcher.Form>
    );
  } else {
    return (
      <span onClick={edit}>
        {hpkeConfig.name || `HPKE Config ${hpkeConfig.contents.id}`}{" "}
        <Button
          variant="outline-secondary"
          onClick={edit}
          size="sm"
          className="ml-auto"
        >
          <PencilFill />
        </Button>
      </span>
    );
  }
}

function RelativeTime({ time, missing }: { time?: string; missing?: string }) {
  return time ? (
    <relative-time datetime={time} format="relative">
      {DateTime.fromISO(time).toLocal().toLocaleString(DateTime.DATETIME_SHORT)}
    </relative-time>
  ) : (
    <>{missing || "never"}</>
  );
}

function DeleteButton({ hpkeConfig }: { hpkeConfig: HpkeConfig }) {
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
            Confirm HPKE Config Deletion {hpkeConfig.name}
          </Modal.Title>
        </Modal.Header>
        <Modal.Body>
          This HPKE Config will immediately be inactivated and cannot be used to
          create new tasks. Existing tasks will continue to function.
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={close}>
            Close
          </Button>
          <fetcher.Form method="delete" action={hpkeConfig.id}>
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

function HpkeConfigRow({ hpkeConfig }: { hpkeConfig: HpkeConfig }) {
  return (
    <tr>
      <td>
        <Name hpkeConfig={hpkeConfig} />
      </td>
      <td>{hpkeConfig.contents.kem_id}</td>
      <td>{hpkeConfig.contents.kdf_id}</td>
      <td>{hpkeConfig.contents.aead_id}</td>
      <td>
        <RelativeTime time={hpkeConfig.created_at} />
      </td>
      <td>
        <DeleteButton hpkeConfig={hpkeConfig} />
      </td>
    </tr>
  );
}
