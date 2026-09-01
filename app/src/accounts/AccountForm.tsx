import { useState, useCallback, ChangeEvent } from "react";
import { Button, FormGroup, FormLabel, FormControl } from "react-bootstrap";
import { Form } from "react-router-dom";
import { BuildingAdd } from "react-bootstrap-icons";

export default function AccountForm() {
  const [name, setName] = useState<string>("");
  const updateName = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setName(event.target.value);
    },
    [setName],
  );

  return (
    <Form action="." method="POST">
      <FormGroup className="mb-3" controlId="Account">
        <FormLabel>Account Name</FormLabel>
        <FormControl
          name="name"
          type="text"
          placeholder="Account Name"
          value={name}
          onChange={updateName}
          data-1p-ignore
        />
      </FormGroup>
      <Button variant="primary" type="submit">
        <BuildingAdd /> Create Account
      </Button>
    </Form>
  );
}
