// import { find } from "lodash";
// import types from "../types";

export const allowedRouteCondition = (authUser, allowedRoles) => {
  // if (!authUser) {
  //   return false;
  // }
  // if (find(authUser.roles, (role) => role === types.UserRoles.Admin)) {
  //   return true;
  // }
  // for (let i = 0; i < allowedRoles.length; i += 1) {
  //   if (find(authUser.roles, (role) => role === allowedRoles[i])) {
  //     return true;
  //   }
  // }
  return false;
};
