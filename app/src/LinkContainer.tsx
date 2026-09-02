import React from "react";
import { useNavigate, useHref, To } from "react-router-dom";

type LinkContainerProps = {
  children: React.ReactElement<LinkProps>;
  to: To;
};

type LinkProps = {
  href?: string;
  onClick?: (e: MouseEvent) => void;
};

/**
 * This React function component is a stripped-down version of LinkContainer
 * from the react-router-bootstrap package.
 *
 * There should be a single child element, a react-bootstrap component that
 * takes href and onClick props. Examples include Button, ListGroup.Item,
 * Breadcrumb.Item, and Nav.Link. This component sets the link's target, and
 * intercepts clicks to perform react-router navigations.
 *
 * Compared to the original react-router-bootstrap implementation, all options
 * we don't use have been stripped out, and TypeScript types have been added. We
 * plan to make further changes to make this compatible with react-router v7,
 * etc.
 */
export function LinkContainer({
  children,
  to,
}: LinkContainerProps): React.ReactElement<LinkProps> {
  const navigate = useNavigate();
  const href = useHref(to);

  const onClick = React.useCallback(
    (event: MouseEvent) => {
      if (children.props.onClick) {
        children.props.onClick(event);
      }

      if (
        !event.defaultPrevented &&
        event.button === 0 &&
        !event.metaKey &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.shiftKey
      ) {
        event.preventDefault();
        navigate(to);
      }
    },
    [children, navigate, to],
  );

  // Assert only one child element was passed in.
  const child = React.Children.only(children);

  return React.cloneElement(child, { href, onClick });
}
