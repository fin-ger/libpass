Feature: Traversing a password store
  Scenario: Traversing level-order over all entries in the password store
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And the password store is traversed in level-order
    Then the passwords and directories are iterated in level-order form

  Scenario: Traversing pre-order over all entries in the password store
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And the password store is traversed in pre-order
    Then the passwords and directories are iterated in pre-order form

  Scenario: Traversing post-order over all entries in the password store
    Given a password store exists
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And the password store is traversed in post-order
    Then the passwords and directories are iterated in post-order form
