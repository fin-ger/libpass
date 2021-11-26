Feature: Git operations in the password store
  Scenario: Commit a new password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    Then the repository is clean and contains a new commit

  Scenario: Commit an edited password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is edited
    Then the repository is clean and contains a new commit

  Scenario: Commit a removed password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is removed
    Then the repository is clean and contains a new commit

  Scenario: Commit a renamed password
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a password is renamed
    Then the repository is clean and contains a new commit

  Scenario: Commit a password in a new directory
    Given a password store exists
    And the password store uses git
    And a password store is opened
    When the password store is successfully opened
    And a directory is created
    And a password is created in the new directory
    Then the repository is clean and contains a new commit

  Scenario: Commit a renamed directory
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a directory is renamed
    Then the repository is clean and contains a new commit

  Scenario: Commit a removed directory
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    And a directory is removed
    Then the repository is clean and contains a new commit

  Scenario: Query the status of an unaltered password store
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And a password store is opened
    When the password store is successfully opened
    Then the repository is clean

  Scenario: Push fast-forward commits to the git remote
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And a password store is opened
    When the password store is successfully opened
    And a password is created
    And the commit is pushed to the remote
    Then pushing the commit succeeds

  Scenario: Push of non-fastforward commits to the git remote fails
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And a password is created
    And the commit is pushed to the remote
    Then pushing the commit fails

  Scenario: Pull fast-forward changes from git remote without interaction
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And changes are pulled from the remote
    Then no conflicts need to be resolved
    And the remote's commits are fast-forwarded
    And the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with automatic merging
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And a new password is created
    And changes are pulled from the remote
    Then no conflicts need to be resolved
    And the remote's commits are merged
    And the repository is clean

  Scenario: Pull non-fast-forward changes from the git remote with manual merging and resolve merge conflict by letting the user resolve it using decrypted passwords
    Given a password store exists
    And the password store uses git
    And passwords are stored in the password store
    And the repository has a remote
    And the repository's remote contains new commits
    And a password store is opened
    When the password store is successfully opened
    And a password is edited
    And changes are pulled from the remote
    Then merge conflicts are manually resolved
    And the remote's commits are merged
    And the repository is clean
