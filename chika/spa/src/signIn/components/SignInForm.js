import React, { useState } from "react";
import PropTypes from "prop-types";
import { withRouter } from "react-router-dom";
import { compose } from "recompose";
import { useDispatch } from "react-redux";

import { HOME } from '../../constants';
import { signIn } from "../../session";

import "./SignInForm.scss";

function SignInForm({ history }) {
  const [formState, setFormState] = useState({
    username: "",
    password: "",
  });
  const { username, password } = formState;
  
  const dispatch = useDispatch();
  const isInvalid = password === "" || username === "";

  const onSubmit = (event) => {
    event.preventDefault();
    dispatch(signIn({ username, password }, (authorized) => {
      if (authorized) {
        history.push(HOME);
      }
    }));
  };

  const onChange = (event) => {
    const name = event.target.name;
    const value = event.target.value;
    setFormState(state => ({...state, ...{ [name]: value }}));
  };

  return (
    <form onSubmit={onSubmit} className="SignInForm">
      <input
        name="username"
        value={username}
        onChange={onChange}
        type="text"
        placeholder="Enter username"
      />
      <input
        name="password"
        value={password}
        onChange={onChange}
        type="password"
        placeholder="Password"
      />
      <button disabled={isInvalid} type="submit">
        Sign In
      </button>
    </form>
  );
}

SignInForm.propTypes = {
  history: PropTypes.shape({
    push: PropTypes.func,
  }),
};

export default compose(withRouter)(SignInForm);
