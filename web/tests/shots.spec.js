import { test } from "@playwright/test";
test.skip(!process.env.SHOTS, "screenshots only with SHOTS=1");
test("screens", async ({ page }, info) => {
  const dir = `../docs/screens/shore-${info.project.name}`;
  await page.goto("/shore/?replay=1&speed=20");
  await page.waitForTimeout(400);
  await page.screenshot({ path: `${dir}-1-start.png`, fullPage: true });
  await page.getByTestId("connect").click();
  await page.getByTestId("fresh").click();
  await page.getByRole("button", { name: "Get quote" }).click();
  await page.getByTestId("approve").waitFor();
  await page.screenshot({ path: `${dir}-2-quote.png`, fullPage: true });
  await page.getByTestId("approve").click();
  await page.waitForTimeout(1800);
  await page.screenshot({ path: `${dir}-3-landing.png`, fullPage: true });
  await page.locator("#readback[data-landed]").waitFor({ timeout: 30000 });
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${dir}-4-readback.png`, fullPage: true });
});
