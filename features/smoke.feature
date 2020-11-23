Feature: Check if the password store blows on first use
  Scenario: A password store is opened
    Given a password store exists at a manually provided location
    And passwords are stored in the password store
    When a password store is opened at a manually provided location
    And the password store is successfully opened
    Then the password store has no errors
    And the password store contains passwords
