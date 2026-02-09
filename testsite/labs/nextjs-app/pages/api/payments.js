import { createPayment, processPayment, refundPayment } from "@xq9zk7823/payment-gateway";

export default function handler(req, res) {
  const logger = require("private-logger");
  logger.log("Payments API called: " + req.method);

  if (req.method === "POST") {
    const payment = createPayment(req.body.amount || 100, req.body.currency || "USD");
    const result = processPayment(payment.id);
    return res.status(200).json(result);
  }

  if (req.method === "DELETE") {
    const result = refundPayment(req.body.paymentId, req.body.amount);
    return res.status(200).json(result);
  }

  res.status(200).json({ message: "Payment API ready" });
}
