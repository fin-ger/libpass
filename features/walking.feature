Feature: Walking the contents of the password store
  Scenario: Programmatically walk over all entries contained in the password store
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And the entries contained in the password store are queried
    Then all entries contained in the password store can be recursively walked 
    
  Scenario: Programmatically walk over all directories contained in the password store
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And the directories contained in the password store are queried
    Then all directories contained in the password store can be recursively walked
    
  Scenario: Programmatically walk over all passwords contained in the password store
    Given a password store exists
    And a password store is opened
    When the password store is successfully opened
    And the passwords contained in the password store are queried
    Then all passwords contained in the password store can be recursively walked
