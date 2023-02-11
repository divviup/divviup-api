import { useState, useCallback, ChangeEvent } from "react";
import Button from "react-bootstrap/Button";
import Form from "react-bootstrap/Form";
import ApiClient from "./ApiClient";
import { useNavigate, Form as RRForm } from "react-router-dom";
import { BuildingAdd } from "react-bootstrap-icons";

export default function AccountForm({ apiClient }: { apiClient: ApiClient }) {
  let [name, setName] = useState<string>("");
  let navigate = useNavigate();
  let create = useCallback(() => {
    if (name) {
      apiClient
        .createAccount({ name })
        .then((account) => navigate(`/accounts/${account.id}`));
    }
  }, [name, apiClient, navigate]);

  let updateName = useCallback(
    (event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
      setName(event.target.value as string);
    },
    [setName]
  );

  return (
    <RRForm onSubmit={create}>
      <Form.Group className="mb-3" controlId="Account">
        <Form.Label>Account Name</Form.Label>
        <Form.Control
          type="text"
          placeholder="Account Name"
          value={name}
          onChange={updateName}
        />
      </Form.Group>
      <Button variant="primary" type="submit">
        <BuildingAdd /> Create Account
      </Button>
    </RRForm>
  );
}
