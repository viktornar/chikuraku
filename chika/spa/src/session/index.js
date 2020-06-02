import { createSlice } from "@reduxjs/toolkit";
import axios from "axios";
import { browserHistory } from 'react-router';

import withAuthorization from "./withAuthorization";
import AuthContext from "./context";

import { AUTH_CREATE_TOKEN_URI, HOME } from "../constants";

const sessionSlice = createSlice({
  name: "session",
  initialState: {
    token: localStorage.getItem("token"),
    me: JSON.parse(localStorage.getItem("me")),
    isLoading: true,
  },
  reducers: {
    startLoading: (state) => {
      state.isLoading = true;
    },
    stopLoading: (state) => {
      state.isLoading = false;
    },
    setToken: (state, action) => {
      localStorage.setItem("token", action.payload);
      state.token = action.payload;
    },
    setMe: (state, action) => {
      localStorage.setItem("me", JSON.stringify(action.payload));
      state.me = action.payload.me;
    },
  },
});

export const fetchMe = () => async (dispatch) => {};

export const signIn = ({ username, password }, callback) => async (dispatch) => {
  try {
    dispatch(startLoading());
    const response = await axios.post(AUTH_CREATE_TOKEN_URI, {
      username,
      password,
    });
    if (response.data) {
      const { token } = response.data;
      dispatch(setToken(token));
      callback(true);
    }
    dispatch(stopLoading());
  } catch (error) {
    dispatch(stopLoading());
    callback(false);
  }
};

export const sessionReducer = sessionSlice.reducer;
export const { setToken, startLoading, stopLoading } = sessionSlice.actions;

export { AuthContext, withAuthorization };
