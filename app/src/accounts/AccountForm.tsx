import { useState, useCallback, ChangeEvent } from "react";
import Button from "react-bootstrap/Button";
import FormGroup from "react-bootstrap/FormGroup";
import FormLabel from "react-bootstrap/FormLabel";
import FormControl from "react-bootstrap/FormControl";
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
        />
      </FormGroup>
      <Button variant="primary" type="submit">
        <BuildingAdd /> Create Account
      </Button>
    </Form>
  );
}
