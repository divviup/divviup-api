import Breadcrumb from "react-bootstrap/Breadcrumb";
import Col from "react-bootstrap/Col";
import Row from "react-bootstrap/Row";
import { LinkContainer } from "react-router-bootstrap";
import React, { Suspense } from "react";
import { Await, useRouteLoaderData } from "react-router-dom";
import { Account } from "./ApiClient";
import Placeholder from "react-bootstrap/Placeholder";
import { Button, OverlayTrigger, Tooltip } from "react-bootstrap";
import {
  ClipboardCheckFill,
  Clipboard,
  BoxArrowUpRight,
} from "react-bootstrap-icons";

export function WithAccount({
  children,
}: {
  children: (data: Awaited<Account>) => React.ReactNode;
}) {
  const { account } = useRouteLoaderData("account") as {
    account: Promise<Account>;
  };

  return <Await resolve={account}>{children}</Await>;
}

export function AccountBreadcrumbs({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Row>
      <Col>
        <Breadcrumb>
          <LinkContainer to="/accounts">
            <Breadcrumb.Item>Accounts</Breadcrumb.Item>
          </LinkContainer>
          <Suspense
            fallback={
              <Breadcrumb.Item>
                <Placeholder size="sm" xs={2} />
              </Breadcrumb.Item>
            }
          >
            <WithAccount>
              {(account) => (
                <LinkContainer to={`/accounts/${account.id}`}>
                  <Breadcrumb.Item>{account.name}</Breadcrumb.Item>
                </LinkContainer>
              )}
            </WithAccount>
          </Suspense>
          {children}
        </Breadcrumb>
      </Col>
    </Row>
  );
}

export function Copy({
  children,
  clipboardContents,
}: {
  children(copy: undefined | (() => void), copied: boolean): React.ReactElement;
  clipboardContents: string;
}) {
  if ("clipboard" in navigator) {
    const [copied, setCopied] = React.useState(false);
    const copy = React.useCallback(() => {
      navigator.clipboard.writeText(clipboardContents).then(() => {
        setCopied(true);
      });
    }, [setCopied, clipboardContents]);

    return (
      <OverlayTrigger
        overlay={<Tooltip>{copied ? "Copied!" : "Click to copy"}</Tooltip>}
      >
        {children(copy, copied)}
      </OverlayTrigger>
    );
  } else {
    return children(undefined, false);
  }
}

export function CopyCode({ code }: { code: string }) {
  return (
    <Copy clipboardContents={code}>
      {(copy, copied) =>
        copy ? (
          <span onClick={copy} style={{ cursor: "pointer" }}>
            <code className="user-select-all">{code}</code>{" "}
            <Button size="sm" variant="outline-secondary" className="ml-auto">
              {copied ? <ClipboardCheckFill /> : <Clipboard />}
            </Button>
          </span>
        ) : (
          <code className="user-select-all">{code}</code>
        )
      }
    </Copy>
  );
}

export function usePromise<T>(promise: PromiseLike<T>, initialState: T): T {
  const [state, setState] = React.useState<T>(initialState);
  React.useEffect(() => {
    promise.then((value) => setState(value));
  }, [promise]);
  return state;
}

export function usePromiseAll<U, T extends unknown[]>(
  promises: [...T],
  then: (arr: { [P in keyof T]: Awaited<T[P]> }) => U | PromiseLike<U>,
  initialState: U,
): U {
  return usePromise(
    React.useMemo(() => Promise.all(promises).then(then), promises),
    initialState,
  );
}

export function OutLink({
  href,
  children,
}: {
  href: string;
  children: React.ReactNode;
}) {
  return (
    <a href={href} target="_blank" rel="noreferrer" className="icon-link">
      {children}
      <BoxArrowUpRight />
    </a>
  );
}
