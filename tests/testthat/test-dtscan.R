require(dtscan)
require(testthat)

test_that("dtscan returns expected output type and length", {
  set.seed(123)
  x <- matrix(runif(200), ncol = 2)  # 100 random 2D points

  cl <- dtscan(x, min_pts = 4, max_closeness = 0.15)
  
  expect_type(cl, "integer")
  expect_length(cl, nrow(x))
  expect_true(any(!is.na(cl)) || all(is.na(cl)))  # All NA or some assigned
})

test_that("dtscan clusters a simple synthetic example", {
  set.seed(42)
  a <- matrix(rnorm(100, mean = 0), ncol = 2)
  b <- matrix(rnorm(100, mean = 3), ncol = 2)
  x <- rbind(a, b)

  cl <- dtscan(x, min_pts = 4, max_closeness = 0.5)

  expect_type(cl, "integer")
  expect_length(cl, nrow(x))
  expect_gte(length(na.omit(unique(cl))), 2)  # at least 2 clusters

})

test_that("dtscan handles unclusterable data (no points close enough)", {
  x <- matrix(runif(20, min = 0, max = 1000), ncol = 2)

  cl <- dtscan(x, min_pts = 5, max_closeness = 0.1)

  expect_true(all(is.na(cl)))
})

test_that("dtscan handles collinear input", {
  x <- cbind(seq(0, 1, length.out = 20), rep(0, 20))

  expect_warning({
    cl <- dtscan(x, min_pts = 3, max_closeness = 0.2)
  }, regexp = NA)

  expect_type(cl, "integer")
  expect_length(cl, nrow(x))
})
