Feature: Initialize a new password store
  Scenario: A new password store is created
    Given no password store exists
    And a new password store is initialized
    When a new password store is successfully created
    Then the password store has no errors
    And the password store is empty
    And the password store's directory exists
    And the password store's directory contains a GPG ID file

  Scenario: A new password store fails to create if it already exists
    Given a password store exists at the default location
    And a new password store is initialized
    Then the initialization of the password store fails
